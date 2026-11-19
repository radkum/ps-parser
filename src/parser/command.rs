use std::{collections::HashMap, sync::LazyLock, vec};

use thiserror_no_std::Error;

use super::{
    SessionScope, StreamMessage, Val,
    value::{PsString, ScriptBlock},
};
use crate::{
    PowerShellSession, ScriptResult,
    parser::{ParserError, over_budget},
};

#[derive(Error, Debug, PartialEq, Clone)]
pub enum CommandError {
    #[error("{0} not found")]
    NotFound(String),
    #[error("Incorrect arguments for method \"{0}\"")]
    IncorrectArgs(String),
    #[error("{0}")]
    ExecutionError(String),
}

impl From<ParserError> for CommandError {
    fn from(value: ParserError) -> CommandError {
        CommandError::ExecutionError(value.to_string())
    }
}
use crate::parser::ParserResult;
pub type CallablePredType<I, O> = Box<dyn Fn(Vec<I>, &mut PowerShellSession) -> ParserResult<O>>;

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub val: Val,                     // Regular return value
    pub deobfuscated: Option<String>, // Message to a specific stream
}

impl CommandOutput {
    pub fn new(val: Val, deobfuscated: Vec<String>) -> Self {
        Self {
            val,
            deobfuscated: if deobfuscated.is_empty() {
                None
            } else {
                Some(deobfuscated.join(crate::NEWLINE))
            },
        }
    }
}

impl From<ScriptResult> for CommandOutput {
    fn from(script_result: ScriptResult) -> Self {
        CommandOutput {
            val: script_result.result().into(),
            deobfuscated: script_result.deobfuscated().into(),
        }
    }
}

impl From<Val> for CommandOutput {
    fn from(val: Val) -> Self {
        CommandOutput {
            val,
            deobfuscated: None,
        }
    }
}
#[derive(Debug)]
pub enum CommandInner {
    Cmdlet(String),
    Path(String),
    ScriptBlock(ScriptBlock),
}

#[derive(Debug)]
pub struct Command {
    command_inner: CommandInner,
    args: Vec<CommandElem>,
    scope: SessionScope,
}

impl Command {
    pub(crate) fn script_block(script_block: ScriptBlock) -> Self {
        Self {
            command_inner: CommandInner::ScriptBlock(script_block),
            args: Vec::new(),
            scope: SessionScope::Current,
        }
    }

    pub(crate) fn cmdlet(cmdlet: &str) -> Self {
        Self {
            command_inner: CommandInner::Cmdlet(cmdlet.to_string()),
            args: Vec::new(),
            scope: SessionScope::Current,
        }
    }

    pub(crate) fn path(path: &str) -> Self {
        Self {
            command_inner: CommandInner::Path(path.to_string()),
            args: Vec::new(),
            scope: SessionScope::Current,
        }
    }

    pub(crate) fn set_session_scope(&mut self, scope: SessionScope) {
        self.scope = scope;
    }

    pub(crate) fn with_args(&mut self, args: Vec<CommandElem>) {
        self.args.extend(args);
    }

    pub(crate) fn name(&self) -> String {
        match &self.command_inner {
            CommandInner::Cmdlet(name) => name.clone(),
            CommandInner::Path(path) => path.clone(),
            CommandInner::ScriptBlock(_) => "ScriptBlock".to_string(),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut command = match &self.command_inner {
            CommandInner::Cmdlet(name) => name.clone(),
            CommandInner::Path(path) => path.clone(),
            CommandInner::ScriptBlock(sb) => sb.deobfuscated_string(),
        };

        if !self.args.is_empty() {
            let args_str = self
                .args
                .iter()
                .map(|arg| arg.display())
                .collect::<Vec<_>>()
                .join(" ");
            command = format!("{} {}", command, args_str);
        }
        write!(f, "{}", command)
    }
}

pub(crate) type FunctionPredType =
    fn(&mut Vec<CommandElem>, &mut PowerShellSession) -> ParserResult<CommandOutput>;

impl Command {
    const COMMAND_MAP: LazyLock<HashMap<&'static str, FunctionPredType>> = LazyLock::new(|| {
        HashMap::from([
            ("write-output", write_output as FunctionPredType),
            ("write-warning", write_warning as FunctionPredType),
            ("write-host", write_host as FunctionPredType),
            ("write-error", write_error as FunctionPredType),
            ("write-verbose", write_verbose as FunctionPredType),
            ("where-object", where_object as FunctionPredType),
            ("get-location", get_location as FunctionPredType),
            ("powershell", powershell as FunctionPredType),
            ("foreach-object", foreach_object as FunctionPredType),
        ])
    });

    pub(crate) fn get(name: &str) -> Option<FunctionPredType> {
        Self::COMMAND_MAP.get(name).cloned()
    }

    fn impl_execute(&mut self, ps: &mut PowerShellSession) -> ParserResult<CommandOutput> {
        match &mut self.command_inner {
            CommandInner::ScriptBlock(sb) => sb.run(self.args.clone(), ps, None),
            CommandInner::Cmdlet(name) => {
                if let Some(fun) = ps.variables.get_function(&name.to_ascii_lowercase()) {
                    fun(self.args.clone(), ps)
                } else if let Some(cmdlet) = Self::get(&name.to_ascii_lowercase()) {
                    cmdlet(&mut self.args, ps)
                } else {
                    Err(ParserError::from(CommandError::NotFound(name.clone())))?
                }
            }
            CommandInner::Path(path) => {
                Err(ParserError::from(CommandError::NotFound(path.clone())))?
            }
        }
    }

    pub(crate) fn execute(&mut self, ps: &mut PowerShellSession) -> ParserResult<CommandOutput> {
        let new_scope = matches!(self.scope, SessionScope::New);

        if new_scope {
            ps.push_scope_session();
        }
        let res = self.impl_execute(ps);
        if new_scope {
            ps.pop_scope_session();
        }
        res
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum CommandElem {
    Parameter(String),
    Argument(Val),
}

impl From<Val> for CommandElem {
    fn from(value: Val) -> Self {
        CommandElem::Argument(value)
    }
}

impl CommandElem {
    pub fn display(&self) -> String {
        match self {
            CommandElem::Parameter(s) => s.clone(),
            CommandElem::Argument(v) => v.cast_to_string(),
        }
    }
}

// Where-Object cmdlet implementation
fn where_object(
    args: &mut Vec<CommandElem>,
    ps: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    log::debug!("args: {:?}", args);
    if args.len() < 2 {
        return Err(CommandError::IncorrectArgs(
            "Where-Object requires at least two arguments".into(),
        )
        .into());
    }

    let CommandElem::Argument(argument) = args[0].clone() else {
        return Err(CommandError::IncorrectArgs(
            "First argument must be an CommandElem::Argument".into(),
        )
        .into());
    };

    let sb = if let CommandElem::Argument(Val::ScriptBlock(sb)) = &args[1] {
        sb
    } else {
        &ScriptBlock::from_command_elements(&args[1..])
    };

    let filtered_elements = if let Val::Array(elements) = argument {
        elements
            .into_iter()
            .filter_map(|element| match sb.run(vec![], ps, Some(element.clone())) {
                Err(er) if over_budget(&er) => Some(Err(er)),
                Err(er) => {
                    ps.errors.push(er);
                    None
                }
                Ok(b) => b.val.cast_to_bool().then_some(Ok(element)),
            })
            .collect::<ParserResult<Vec<_>>>()?
    } else if sb
        .run(vec![], ps, Some(argument.clone()))?
        .val
        .cast_to_bool()
    {
        vec![argument.clone()]
    } else {
        vec![]
    };

    let val = if filtered_elements.is_empty() {
        Val::Null
    } else if filtered_elements.len() == 1 {
        filtered_elements[0].to_owned()
    } else {
        Val::Array(filtered_elements)
    };

    Ok(CommandOutput {
        val,
        deobfuscated: None,
    })
}

// Foreach-Object cmdlet implementation
fn foreach_object(
    args: &mut Vec<CommandElem>,
    ps: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    log::debug!("args: {:?}", args);
    if args.len() != 2 {
        return Err(CommandError::IncorrectArgs(
            "Foreach-Object requires exactly two arguments".into(),
        )
        .into());
    }

    let CommandElem::Argument(argument) = args[0].clone() else {
        return Err(CommandError::IncorrectArgs(
            "First argument must be an CommandElem::Argument".into(),
        )
        .into());
    };

    let CommandElem::Argument(Val::ScriptBlock(sb)) = &args[1] else {
        return Err(
            CommandError::IncorrectArgs("Second argument must be a script block".into()).into(),
        );
    };

    let transformed_elements = if let Val::Array(elements) = argument {
        elements
            .into_iter()
            .map(|element| match sb.run(vec![], ps, Some(element.clone())) {
                Err(er) if over_budget(&er) => Err(er),
                Err(er) => {
                    ps.errors.push(er);
                    Ok(Val::Null)
                }
                Ok(b) => Ok(b.val),
            })
            .collect::<ParserResult<Vec<_>>>()?
    } else {
        vec![sb.run(vec![], ps, Some(argument))?.val]
    };

    let val = if transformed_elements.is_empty() {
        Val::Null
    } else if transformed_elements.len() == 1 {
        transformed_elements[0].to_owned()
    } else {
        Val::Array(transformed_elements)
    };

    Ok(CommandOutput {
        val,
        deobfuscated: None,
    })
}

fn get_location(
    _args: &mut Vec<CommandElem>,
    _: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    let Ok(dir) = std::env::current_dir() else {
        return Err(CommandError::ExecutionError(
            "Failed to get current directory".into(),
        ))?;
    };

    Ok(CommandOutput {
        val: Val::String(dir.display().to_string().into()),
        deobfuscated: Some(format!("Get-Location \"{}\"", dir.display())),
    })
}
// Helper function to extract message from command arguments
fn extract_message(args: &[CommandElem]) -> String {
    let mut output = Vec::new();
    let mut skip = 0;
    for i in args.iter() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match i {
            CommandElem::Parameter(s) => {
                if s.to_ascii_lowercase().as_str() == "-foregroundcolor" {
                    skip = 1
                } else {
                    output.push(s.clone());
                }
            }
            CommandElem::Argument(val) => {
                output.push(val.display());
            }
        }
    }
    output.join(" ")
}
// Write-Host cmdlet implementation (goes directly to console, not capturable)
fn write_host(
    args: &mut Vec<CommandElem>,
    ps: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    let message = extract_message(args);
    let deobfuscated = format!(
        "Write-Host {}",
        args.iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join(" ")
    );

    ps.add_output_statement(StreamMessage::success(message));
    Ok(CommandOutput {
        val: Val::Null,
        deobfuscated: Some(deobfuscated),
    })
}
// Write-Output cmdlet implementation
fn write_output(
    args: &mut Vec<CommandElem>,
    _: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    let message = extract_message(args);
    let deobfuscated = format!(
        "Write-Output {}",
        args.iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join(" ")
    );

    Ok(CommandOutput {
        val: Val::String(message.clone().into()),
        deobfuscated: Some(deobfuscated),
    })
}

// Write-Warning cmdlet implementation (mimics PowerShell's Write-Warning)
fn write_warning(
    args: &mut Vec<CommandElem>,
    _: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    let message = extract_message(args);
    let deobfuscated = format!(
        "Write-Warning {}",
        args.iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join(" ")
    );

    Ok(CommandOutput {
        val: Val::String(message.clone().into()),
        deobfuscated: Some(deobfuscated),
    })
}

// Write-Error cmdlet implementation
fn write_error(
    args: &mut Vec<CommandElem>,
    _: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    let message = extract_message(args);
    let deobfuscated = format!(
        "Write-Error {}",
        args.iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join(" ")
    );

    Ok(CommandOutput {
        val: Val::String(message.clone().into()),
        deobfuscated: Some(deobfuscated),
    })
}

// Write-Verbose cmdlet implementation
fn write_verbose(
    args: &mut Vec<CommandElem>,
    _: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    let message = extract_message(args);
    let deobfuscated = format!(
        "Write-Verbose {}",
        args.iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join(" ")
    );
    Ok(CommandOutput {
        val: Val::String(message.clone().into()),
        deobfuscated: Some(deobfuscated),
    })
}

// tells the caller that the tokens of an encoded command are missing
pub(crate) fn not_decoded() -> ParserError {
    CommandError::ExecutionError("Encoded command not decoded".into()).into()
}

// Powershell cmdlet implementation. It don't actually invoke a new PowerShell
// process, only deobfuscates the command.
fn powershell(
    args: &mut Vec<CommandElem>,
    ps: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    fn deobfuscate_command(
        args: &mut Vec<CommandElem>,
        ps: &mut PowerShellSession,
    ) -> ParserResult<()> {
        use base64::prelude::*;

        use crate::parser::{
            DECODE_DEPTH, DECODED, NESTED_DECODES, Token, charge, eval_step, fits,
        };

        // a parameter name without its -, /, en dash, em dash or horizontal bar
        fn param_name(p: &str) -> Option<String> {
            let name = p.strip_prefix(['-', '/', '\u{2013}', '\u{2014}', '\u{2015}'])?;
            (!name.is_empty()).then(|| name.to_ascii_lowercase())
        }

        // parameters taking the next argument unvalidated, e.g. -ExecutionPolicy -c
        fn takes_value(name: &str) -> bool {
            name == "ep"
                || name.len() >= 2 && "executionpolicy".starts_with(name)
                || name.len() >= 3 && "configurationname".starts_with(name)
                || name.len() >= 3 && "inputformat".starts_with(name)
                || ["windowstyle", "outputformat", "version", "psconsolefile"]
                    .iter()
                    .any(|p| p.starts_with(name))
        }

        fn is_powershell(elem: Option<&CommandElem>) -> bool {
            let Some(CommandElem::Argument(Val::ScriptText(s) | Val::String(PsString(s)))) = elem
            else {
                return false;
            };
            let name = s
                .rsplit(['\\', '/'])
                .next()
                .unwrap_or(s)
                .to_ascii_lowercase();
            matches!(
                name.strip_suffix(".exe").unwrap_or(&name),
                "powershell" | "pwsh"
            )
        }

        let mut index_to_decode = vec![];
        let mut skip_value = false;
        for (i, elem) in args.iter().enumerate() {
            if std::mem::take(&mut skip_value) {
                continue;
            }
            let (CommandElem::Parameter(s) | CommandElem::Argument(Val::ScriptText(s))) = elem
            else {
                continue;
            };
            let Some(name) = param_name(s) else {
                continue;
            };
            if name == "ec" || "encodedcommand".starts_with(&name) {
                index_to_decode.push(i + 1);
                skip_value = true;
            } else if takes_value(&name) {
                skip_value = true;
            } else if "command".starts_with(&name) {
                // what follows -Command is the command, unless a child powershell
                if !is_powershell(args.get(i + 1)) {
                    break;
                }
                skip_value = true;
            } else if "file".starts_with(&name) {
                break;
            }
        }
        for &i in &index_to_decode {
            if let Some(elem) = args.get_mut(i - 1) {
                *elem = CommandElem::Parameter("-command".to_string());
            }
        }
        let mut args = args.iter_mut().map(Some).collect::<Vec<_>>();

        // -enc officially carries UTF-16LE, but malware often encodes plain ASCII
        fn decode_encoded_command(bytes: &[u8]) -> String {
            // any ASCII char in UTF-16LE puts a 0 at an odd index
            let utf16 = if bytes.len().is_multiple_of(2) {
                bytes.contains(&0)
            } else {
                bytes.iter().skip(1).step_by(2).any(|b| *b == 0)
            };
            if utf16 {
                let units = bytes
                    .chunks_exact(2)
                    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect::<Vec<u16>>();
                return String::from_utf16_lossy(&units);
            }
            String::from_utf8_lossy(bytes).into_owned()
        }

        struct Nested;
        impl Nested {
            fn enter() -> Self {
                DECODE_DEPTH.set(DECODE_DEPTH.get() + 1);
                Nested
            }
        }
        impl Drop for Nested {
            fn drop(&mut self) {
                DECODE_DEPTH.set(DECODE_DEPTH.get().saturating_sub(1));
            }
        }

        // the longest command line a real child process gets
        const MAX_RUNTIME_LEN: usize = 32_767;
        // a literal shrinks per level; a runtime-built command can rebuild itself
        const MAX_NESTED_RUNTIME_DECODES: u32 = 64;

        for i in index_to_decode {
            eval_step()?;
            let (s, literal) = match args.get_mut(i) {
                Some(Some(CommandElem::Argument(Val::ScriptText(s)))) => (s, true),
                Some(Some(CommandElem::Argument(Val::String(PsString(s))))) => (s, false),
                _ => {
                    ps.errors.push(not_decoded());
                    continue;
                }
            };
            if !literal {
                // an identical command was decoded in this scan already; its tokens are in
                if DECODED.with_borrow(|seen| seen.contains(s.as_str())) {
                    continue;
                }
                let nested = DECODE_DEPTH.get() > 0;
                if s.len() > MAX_RUNTIME_LEN
                    || nested && NESTED_DECODES.get() >= MAX_NESTED_RUNTIME_DECODES
                    || !charge(s.len())
                {
                    ps.errors.push(not_decoded());
                    continue;
                }
                DECODED.with_borrow_mut(|seen| seen.insert(s.clone()));
                if nested {
                    NESTED_DECODES.set(NESTED_DECODES.get() + 1);
                }
            }
            // the base64 copy, the decoded bytes and the text
            if !fits(s.len().saturating_mul(4)) {
                ps.errors.push(not_decoded());
                continue;
            }
            let Ok(decoded_bytes) =
                BASE64_STANDARD.decode(s.split_ascii_whitespace().collect::<String>())
            else {
                ps.errors.push(not_decoded());
                continue;
            };
            let decoded_str = decode_encoded_command(&decoded_bytes);
            // a child process sees the environment only, not the parent's variables
            let mut nested = PowerShellSession::new().with_variables(ps.variables.child());
            let parsed = {
                let _nested = Nested::enter();
                nested.parse_script(&decoded_str)
            };
            if let Ok(mut script_result) = parsed {
                let moved = ps.tokens.append(script_result.tokens_mut());
                let deobfuscated = script_result.deobfuscated();
                // nested errors stay inside; only a stopped or undecoded nested run is reported
                let stopped = script_result.errors_ref().iter().any(|e| {
                    over_budget(e)
                        || matches!(e, ParserError::NestingTooDeep | ParserError::Panic(_))
                        || *e == not_decoded()
                });
                if stopped
                    || !charge(
                        moved
                            .saturating_mul(size_of::<Token>())
                            .saturating_add(deobfuscated.len()),
                    )
                {
                    ps.errors.push(not_decoded());
                }
                *s = if deobfuscated.is_empty() {
                    decoded_str
                } else {
                    deobfuscated
                };
                eval_step()?;
            } else {
                log::warn!("Failed to deobfuscate: {}", &decoded_str);
                ps.errors.push(not_decoded());
                *s = decoded_str;
            }
        }
        Ok(())
    }

    deobfuscate_command(args, ps)?;

    Err(CommandError::ExecutionError(
        "Powershell invocation is not supported".into(),
    ))?
}

#[cfg(test)]
mod tests {
    use crate::{NEWLINE, PowerShellSession, PsValue, Variables};

    #[test]
    fn test_where_object() {
        let mut p = PowerShellSession::new();
        let input = r#"$numbers = 1..10;$evenNumbers = $numbers | Where-Object { $_ % 2 -eq 0 };$evenNumbers"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(
            s.result().to_string(),
            vec!["2", "4", "6", "8", "10"].join(NEWLINE)
        );

        let input = r#"5 | where-object {$_ -eq 5}"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(s.result(), PsValue::Int(5));

        let input = r#"5,4 | where-object {$_ -eq 5}"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(s.result(), PsValue::Int(5));

        let input = r#"5,4 | where {$_ -gt 3}"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(
            s.result(),
            PsValue::Array(vec![PsValue::Int(5), PsValue::Int(4)])
        );

        let input = r#"5,4 | where {$_ -lt 3}"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(s.result(), PsValue::Null);

        let input = r#"@(@{val = 4},@{val = 3}) | where val -lt 4"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(
            s.result(),
            PsValue::HashTable(std::collections::HashMap::from([(
                "val".to_string(),
                PsValue::Int(3)
            )]))
        );
    }

    #[test]
    fn test_foreach_object() {
        let mut p = PowerShellSession::new();
        let input = r#"1..5 | foreach { $_ *2 }"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(
            s.result().to_string(),
            vec!["2", "4", "6", "8", "10"].join(NEWLINE)
        );

        let input = r#"5 | % {$_ + 5}"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(s.result(), PsValue::Int(10));

        let input = r#"5,4 | foreach {$_ /2}"#;
        let s = p.parse_script(input).unwrap();
        assert_eq!(
            s.result(),
            PsValue::Array(vec![PsValue::Float(2.5), PsValue::Int(2)])
        );
    }

    #[test]
    fn test_write_output() {
        // assign not existing value, without forcing evaluation
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" $global:var = $env:programfiles; Write-output $var"#;
        let script_res = p.parse_script(input).unwrap();

        assert_eq!(
            script_res.result(),
            PsValue::String(std::env::var("PROGRAMFILES").unwrap())
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                format!(
                    "$global:var = \"{}\"",
                    std::env::var("PROGRAMFILES").unwrap()
                ),
                format!("\"{}\"", std::env::var("PROGRAMFILES").unwrap())
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_res.output(), std::env::var("PROGRAMFILES").unwrap());
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn cmdlets() {
        let mut p = PowerShellSession::new();
        let input = r#""Execution Policy: $(Get-ExecutionPolicy)"
"Current Location: $(Get-Location)""#;
        let s = p.parse_script(input).unwrap();

        // Get-ExecutionPolicy is built-in function
        assert_eq!(
            s.deobfuscated().trim(),
            vec![
                "\"Execution Policy: $(Get-ExecutionPolicy)\"",
                &format!(
                    "\"Current Location: {}\"",
                    std::env::current_dir().unwrap().display()
                )
            ]
            .join(NEWLINE)
        );
    }

    #[test]
    fn param_from_var() {
        let mut p = PowerShellSession::new();
        let input = r#"$x = "Process";Get-ExecutionPolicy -Scope $x"#;
        let s = p.parse_script(input).unwrap();

        // Get-ExecutionPolicy is built-in function
        assert_eq!(
            s.deobfuscated().trim(),
            vec!["$x = \"Process\"", "Get-ExecutionPolicy -scope Process",].join(NEWLINE)
        );
    }

    #[test]
    fn double_quoted_string() {
        let mut p = PowerShellSession::new();
        let input = r#"$x = 5;$y = 3;$result = "Sum: $($x + $y)""#;
        let s = p.parse_script(input).unwrap();

        // Get-ExecutionPolicy is built-in function
        assert_eq!(
            s.deobfuscated().trim(),
            vec!["$x = 5", "$y = 3", "$result = \"Sum: 8\"",].join(NEWLINE)
        );
    }

    #[test]
    fn encoded_command() {
        let mut p = PowerShellSession::new();
        let input = r#"powershell.exe -encodedc VwByAGkAdABlAC0ASABvAHMAdAAgACIAdAB3AGUAZQB0ACwAIAB0AHcAZQBlAHQAIQAiAA=="#;
        let s = p.parse_script(input).unwrap();

        assert_eq!(
            s.deobfuscated().trim(),
            vec![r#"powershell -command Write-Host "tweet, tweet!""#,].join(NEWLINE)
        );
    }

    #[test]
    fn encoded_command2() {
        let mut p = PowerShellSession::new();
        let input = r#"powershell.exe -e JgAgACgAZwBjAG0AIAAoACcAaQBlAHsAMAB9ACcAIAAtAGYAIAAnAHgAJwApACkAIAAoACIAVwByACIAKwAiAGkAdAAiACsAIgBlAC0ASAAiACsAIgBvAHMAdAAgACcASAAiACsAIgBlAGwAIgArACIAbABvACwAIABmAHIAIgArACIAbwBtACAAUAAiACsAIgBvAHcAIgArACIAZQByAFMAIgArACIAaAAiACsAIgBlAGwAbAAhACcAIgApAA=="#;
        let s = p.parse_script(input).unwrap();

        assert_eq!(
            s.deobfuscated().trim(),
            vec![r#"powershell -command gcm iex Write-Host 'Hello, from PowerShell!'"#,]
                .join(NEWLINE)
        );
    }

    #[test]
    fn encoded_command_keeps_outer_and_adds_decoded_tokens() {
        use crate::Token;

        let mut p = PowerShellSession::new();
        // Write-Output 'AmsiUtils'
        let input = "$x = 'outer'; powershell -enc \
                     VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIAAnAEEAbQBzAGkAVQB0AGkAbABzACcA; Get-Date";
        let s = p.parse_script(input).unwrap();

        let tokens = s.tokens();
        let strings = tokens.lowercased_string_set();
        assert!(strings.contains("outer"), "{strings:?}");
        assert!(strings.contains("amsiutils"), "{strings:?}");
        assert!(
            tokens
                .all()
                .iter()
                .any(|t| matches!(t, Token::Command(c) if c.name() == "Get-Date"))
        );
        assert!(
            tokens.methods_and_commands().contains("Write-Output"),
            "{:?}",
            tokens.methods_and_commands()
        );
    }

    #[test]
    fn encoded_command_built_at_runtime_is_decoded() {
        let mut p = PowerShellSession::new();
        let input = "$body = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('\
                     JHMgPSAiQW1zaVNjYW5CdWZmZXIiOyAicWEtYzIybSAkcyI=')) + \" # r_$tag\"; $enc = \
                     [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($body)); \
                     powershell.exe -NoProfile -EncodedCommand $enc";
        let s = p.parse_script(input).unwrap();
        let strings = s.tokens().lowercased_string_set();
        assert!(strings.contains("amsiscanbuffer"), "{strings:?}");
        assert!(
            s.deobfuscated()
                .contains("-command $s = \"AmsiScanBuffer\""),
            "{}",
            s.deobfuscated()
        );
    }

    #[test]
    fn encoded_command_from_unicode_get_bytes_is_decoded() {
        let mut p = PowerShellSession::new();
        let input = "$enc = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes('\
                     Invoke-Mimikatz -Target lsass')); powershell -enc $enc";
        let tokens = p.parse_script(input).unwrap().tokens();
        assert!(
            tokens.methods_and_commands().contains("Invoke-Mimikatz"),
            "{:?}",
            tokens.methods_and_commands()
        );
        assert!(tokens.lowercased_string_set().contains("lsass"));
    }

    #[test]
    fn an_encoded_command_that_is_not_decoded_is_reported() {
        let not_decoded = |input: &str| {
            PowerShellSession::new()
                .parse_script(input)
                .unwrap()
                .errors()
                .iter()
                .any(|e| e.to_string() == "CommandError: Encoded command not decoded")
        };
        // 'unterminated, and powershell -enc of it
        for input in [
            "powershell -enc $missing",
            "powershell -enc JwB1AG4AdABlAHIAbQBpAG4AYQB0AGUAZAA=",
            "powershell -enc cABvAHcAZQByAHMAaABlAGwAbAAgAC0AZQBuAGMAIABKAHcAQgAxAEEARwA0AEEAZABBAEIAbABBAEgASQBBAGIAUQBCAHAAQQBHADQAQQBZAFEAQgAwAEEARwBVAEEAWgBBAEEAPQA=",
        ] {
            assert!(not_decoded(input), "{input}");
        }
        assert!(!not_decoded(
            "powershell -enc VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIAAnAEEAbQBzAGkAVQB0AGkAbABzACcA"
        ));
    }

    const NOT_DECODED: &str = "CommandError: Encoded command not decoded";

    fn decodes_amsiutils(input: &str) -> bool {
        let s = PowerShellSession::new().parse_script(input).unwrap();
        s.tokens().lowercased_string_set().contains("amsiutils")
            && !s.errors().iter().any(|e| e.to_string() == NOT_DECODED)
    }

    #[test]
    fn self_re_encoding_commands_stop_within_the_limits() {
        // decoded once, its copies are the same command
        let inv = "powershell -enc                    ([Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes(($s -f                    [char]39,$s))))";
        for n in [2, 3, 8] {
            let invs = vec![inv; n].join("; ");
            let input = format!("$s='$s={{0}}{{1}}{{0}}; {invs}; Write-Output amsiutils'; {invs}");
            let start = std::time::Instant::now();
            assert!(decodes_amsiutils(&input), "{n}");
            assert!(start.elapsed().as_secs() < 5, "{n}: {:?}", start.elapsed());
        }
        // a different command on every level stops at the nested decode limit
        let inv = "powershell -enc                    ([Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes(($s -f                    [char]39,$s,($n+1)))))";
        let input = format!("$n=0; $s='$n={{2}}; $s={{0}}{{1}}{{0}}; {inv}; {inv}'; {inv}; {inv}");
        let start = std::time::Instant::now();
        let s = PowerShellSession::new().parse_script(&input).unwrap();
        assert!(start.elapsed().as_secs() < 5, "{:?}", start.elapsed());
        assert!(s.errors().iter().any(|e| e.to_string() == NOT_DECODED));
    }

    #[test]
    fn a_nested_command_past_the_memory_budget_keeps_its_tokens() {
        // Write-Output 'AmsiUtils'; 'a'*16000000; 'a'*16000000; 'a'*16000000
        let s = PowerShellSession::new()
            .parse_script(
                "powershell -e \
                 VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIAAnAEEAbQBzAGkAVQB0AGkAbABzACcAOwAgACcAYQAnACoAMQA2ADAAMAAwADAAMAAwADsAIAAnAGEAJwAqADEANgAwADAAMAAwADAAMAA7ACAAJwBhACcAKgAxADYAMAAwADAAMAAwADAA",
            )
            .unwrap();
        assert!(s.tokens().lowercased_string_set().contains("amsiutils"));
        assert!(s.errors().iter().any(|e| e.to_string() == NOT_DECODED));
    }

    #[test]
    fn junk_encoded_commands_do_not_hide_the_payload() {
        // Write-Output 'AmsiUtils'
        let b64 = "VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIAAnAEEAbQBzAGkAVQB0AGkAbABzACcA";
        let junk = "([Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes(\"x$_\")))";
        for input in [
            format!("1..100 | % {{ powershell -e QQA= }}; powershell -e {b64}"),
            format!("powershell{} -e {b64}", " -e QQA=".repeat(100)),
            format!("1..100 | % {{ powershell -e {junk} }}; powershell -e {b64}"),
            format!(
                "1..100 | % {{ powershell -e {junk} }}; powershell -e \
                 ([Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes(\"Write-Output \
                 'AmsiUtils'\")))"
            ),
        ] {
            assert!(decodes_amsiutils(&input), "{input}");
        }
    }

    #[test]
    fn literal_encoded_commands_are_decoded_at_any_depth() {
        use base64::prelude::*;
        let mut input = "Write-Output 'AmsiUtils'".to_string();
        for _ in 0..12 {
            input = format!("powershell -enc {}", BASE64_STANDARD.encode(&input));
        }
        assert!(decodes_amsiutils(&input));
    }

    #[test]
    fn a_runtime_built_command_longer_than_a_command_line_is_not_decoded() {
        let s = PowerShellSession::new()
            .parse_script("$e = 'QUFB' * 10000; powershell -e $e; & ('Get-Pass'+'Hashes')")
            .unwrap();
        assert!(s.errors().iter().any(|e| e.to_string() == NOT_DECODED));
        assert!(s.deobfuscated().contains("Get-PassHashes"));
    }

    #[test]
    fn arguments_of_the_command_are_not_powershell_parameters() {
        for input in [
            r"powershell -c robocopy C:\a D:\b /E",
            "powershell -c Write-Output /e Get-Date",
            "powershell -File x.ps1 -e QQA=",
        ] {
            let s = PowerShellSession::new().parse_script(input).unwrap();
            assert!(
                !s.errors().iter().any(|e| e.to_string() == NOT_DECODED),
                "{input}"
            );
        }
        let s = PowerShellSession::new()
            .parse_script("powershell -c Write-Output /e Get-Date")
            .unwrap();
        assert!(
            s.deobfuscated().contains("/e Get-Date"),
            "{}",
            s.deobfuscated()
        );
    }

    #[test]
    fn a_child_powershell_after_command_keeps_its_parameters() {
        // Write-Output 'AmsiUtils'
        let b64 = "VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIAAnAEEAbQBzAGkAVQB0AGkAbABzACcA";
        for prefix in [
            "powershell -c powershell -e",
            "powershell.exe -NoP -NonI -W Hidden -c powershell.exe -enc",
            r"powershell -c C:\Windows\System32\WindowsPowerShell\v1.0\PowerShell.EXE -e",
            "powershell -c pwsh -e",
            "powershell -ExecutionPolicy -c -e",
            "powershell -ep /file -e",
            "powershell -ExecutionPolicy -Command -EncodedCommand",
            "pwsh -i -e",
            "pwsh -in -e",
        ] {
            assert!(decodes_amsiutils(&format!("{prefix} {b64}")), "{prefix}");
        }
    }

    #[test]
    fn a_not_decoded_command_in_a_script_block_stays_reported() {
        let inner = "$I = '1..70 | % { powershell -enc \
                     ([Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes(\"Write-Output \
                     $_\"))) }'; powershell -enc \
                     ([Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($I)))";
        for input in [
            format!("$sb = {{ {inner} }}; & $sb"),
            format!("function f {{ {inner} }}; f"),
            format!("1 | % {{ {inner} }}"),
            format!("Invoke-Command {{ {inner} }}"),
        ] {
            let s = PowerShellSession::new().parse_script(&input).unwrap();
            let reported = s
                .errors()
                .iter()
                .filter(|e| e.to_string() == NOT_DECODED)
                .count();
            assert!(reported >= 1, "{input}");
        }
    }

    #[test]
    fn an_odd_length_ascii_payload_with_a_nul_is_decoded() {
        // Write-Output 'AmsiUtils' and a NUL
        assert!(decodes_amsiutils(
            "powershell -e V3JpdGUtT3V0cHV0ICdBbXNpVXRpbHMnAA=="
        ));
    }

    #[test]
    fn an_odd_length_utf16_payload_of_mostly_non_ascii_is_decoded() {
        use base64::prelude::*;
        let script = format!("Write-Output 'AmsiUtils';<#{}#>", "\u{4e2d}".repeat(40));
        let mut bytes = script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<u8>>();
        bytes.push(b'A');
        assert!(decodes_amsiutils(&format!(
            "powershell -e {}",
            BASE64_STANDARD.encode(bytes)
        )));
    }

    #[test]
    fn encoded_commands_do_not_copy_the_outer_variables() {
        let input = format!(
            "$big = 'a' * 16000000; powershell{}",
            " -e QQA=".repeat(1000)
        );
        let start = std::time::Instant::now();
        PowerShellSession::new().parse_script(&input).unwrap();
        assert!(start.elapsed().as_secs() < 3, "{:?}", start.elapsed());

        // Write-Output $e; Write-Output $env:x
        let vars = Variables::from_ini_string("[env]\nx = amsiutils").unwrap();
        let s = PowerShellSession::new()
            .with_variables(vars)
            .parse_script(
                "$e = 'outer'; powershell -enc VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIAAkAGUA; \
                 powershell -enc VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIAAkAGUAbgB2ADoAeAA=",
            )
            .unwrap();
        let deobfuscated = s.deobfuscated();
        assert!(deobfuscated.contains("-command \"$e\""), "{deobfuscated}");
        assert!(
            deobfuscated.contains("-command \"amsiutils\""),
            "{deobfuscated}"
        );
    }

    #[test]
    fn a_stopped_nested_command_reports_only_that_it_was_not_decoded() {
        // function f { f }; f; & ('Invoke-Mimi'+'katz')
        let s = PowerShellSession::new()
            .parse_script(
                "Write-Output 'VirtualAlloc'; powershell -enc \
                 ZgB1AG4AYwB0AGkAbwBuACAAZgAgAHsAIABmACAAfQA7ACAAZgA7ACAAJgAgACgAJwBJAG4AdgBvAGsAZQAtAE0AaQBtAGkAJwArACcAawBhAHQAegAnACkA",
            )
            .unwrap();
        let errors = s.errors().iter().map(|e| e.to_string()).collect::<Vec<_>>();
        assert!(errors.iter().any(|e| e == NOT_DECODED), "{errors:?}");
        assert!(
            !errors
                .iter()
                .any(|e| e == "Nesting too deep" || e == "Skip" || e.starts_with("Panic")),
            "{errors:?}"
        );
    }

    #[test]
    fn odd_lossy_and_spaced_payloads_are_decoded() {
        for input in [
            // UTF-16LE Write-Output amsiutils, one byte more
            "powershell -enc VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIABhAG0AcwBpAHUAdABpAGwAcwBB",
            // and with a lone surrogate after a comment
            "powershell -enc VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIABhAG0AcwBpAHUAdABpAGwAcwAgACMAANg=",
            "powershell -enc \"VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIABhAG0AcwBpAHUAd\nABpAGwAcwBB \"",
        ] {
            assert!(decodes_amsiutils(input), "{input}");
        }
    }

    #[test]
    fn encoded_command_parameter_spellings() {
        // Write-Output 'AmsiUtils'
        let b64 = "VwByAGkAdABlAC0ATwB1AHQAcAB1AHQAIAAnAEEAbQBzAGkAVQB0AGkAbABzACcA";
        for param in [
            "-e",
            "-ec",
            "-EncodedCommand",
            "/enc",
            "/e",
            "/EncodedCommand",
            "\u{2013}enc",
            "\u{2014}e",
            "\u{2015}enc",
        ] {
            assert!(
                decodes_amsiutils(&format!("powershell {param} {b64}")),
                "{param}"
            );
        }
        for param in ["-ex", "-ExecutionPolicy"] {
            let s = PowerShellSession::new()
                .parse_script(&format!("powershell {param} Bypass -c Get-Date"))
                .unwrap();
            assert!(
                !s.errors().iter().any(|e| e.to_string() == NOT_DECODED),
                "{param}"
            );
        }
    }

    #[test]
    fn unknown_variable_argument_keeps_the_command() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = "Register-ScheduledTask -TaskName 'Upd' -Action $action; foreach ($i in 1..2) \
                     { New-Item -Name \"x$i\" }; Get-Item ($missing + 1)";
        let commands = p.parse_script(input).unwrap().tokens().commands();
        let args = |name: &str| {
            commands
                .iter()
                .find(|c| c.name() == name)
                .map(|c| c.args().clone())
        };
        assert_eq!(
            args("Register-ScheduledTask").unwrap(),
            ["-taskname", "Upd", "-action", "$action"]
        );
        assert!(args("New-Item").is_some(), "{commands:?}");
        assert!(args("Get-Item").is_some(), "{commands:?}");
    }

    #[test]
    fn commands_in_unevaluated_statement_blocks() {
        let input = "try { New-Item -Name a } catch { Remove-Item b } finally { Get-Item c }; \
                     while ($true) { Set-Item d }; do { Clear-Item e } until ($true); for ($i = \
                     0; $i -lt 2; $i++) { Move-Item f }; $sb = { try { Copy-Item g } catch {} }";
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let commands = p.parse_script(input).unwrap().tokens().commands();
        for name in [
            "New-Item",
            "Remove-Item",
            "Get-Item",
            "Set-Item",
            "Clear-Item",
            "Move-Item",
            "Copy-Item",
        ] {
            assert!(
                commands.iter().any(|c| c.name() == name),
                "{name}: {commands:?}"
            );
        }
    }

    #[test]
    fn call_operator_with_a_string_name_invokes_the_command() {
        let invoke = |input: &str| {
            let s = PowerShellSession::new().parse_script(input).unwrap();
            let commands = s
                .tokens()
                .methods_and_commands()
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>();
            (s.deobfuscated(), commands)
        };

        let (deobfuscated, commands) = invoke(
            r#"function Invoke-QaTest { "body-ran" }; & ("Invoke-Qa" + "Test"); . 'Invoke-QaTest'"#,
        );
        assert_eq!(commands, ["Invoke-QaTest"]);
        assert_eq!(
            deobfuscated,
            [
                "function Invoke-QaTest { \"body-ran\" }",
                "\"body-ran\"",
                "\"body-ran\""
            ]
            .join(NEWLINE)
        );

        let (_, commands) = invoke(r#"& "C:\Program Files\x.exe" -a 1"#);
        assert_eq!(commands, [r"C:\Program Files\x.exe"]);
    }

    #[test]
    fn encoded_command3() {
        let mut p = PowerShellSession::new();
        let input = r#"& (gcm ('ie{0}' -f 'x')) ("Wr"+"it"+"e-H"+"ost 'H"+"el"+"lo, fr"+"om P"+"ow"+"erS"+"h"+"ell!'")"#;
        let s = p.parse_script(input).unwrap();

        assert_eq!(
            s.deobfuscated().trim(),
            vec![r#"gcm iex Write-Host 'Hello, from PowerShell!'"#,].join(NEWLINE)
        );
    }
}
