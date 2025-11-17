use std::{collections::HashMap, sync::LazyLock, vec};

use thiserror_no_std::Error;

use super::{SessionScope, StreamMessage, Val, value::ScriptBlock};
use crate::{PowerShellSession, ScriptResult, parser::ParserError};

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
pub type CallablePredType =
    Box<dyn Fn(Vec<CommandElem>, &mut PowerShellSession) -> ParserResult<CommandOutput>>;

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

    pub(crate) fn args(&self) -> Vec<String> {
        self.args.iter().map(|arg| arg.display()).collect()
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
    #[allow(dead_code)]
    ArgList(String),
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
            CommandElem::ArgList(s) => s.clone(),
        }
    }
}

// Where-Object cmdlet implementation
fn where_object(
    args: &mut Vec<CommandElem>,
    ps: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    log::debug!("args: {:?}", args);

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
            .iter()
            .filter(|&element| match sb.run(vec![], ps, Some(element.clone())) {
                Err(er) => {
                    ps.errors.push(er);
                    false
                }
                Ok(b) => b.val.cast_to_bool(),
            })
            .cloned()
            .collect::<Vec<_>>()
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
                Err(er) => {
                    ps.errors.push(er);
                    Val::Null
                }
                Ok(b) => b.val,
            })
            .collect::<Vec<_>>()
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
            CommandElem::ArgList(_) => {}
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

// Powershell cmdlet implementation. It don't actually invoke a new PowerShell
// process, only deobfuscates the command.
fn powershell(
    args: &mut Vec<CommandElem>,
    ps: &mut PowerShellSession,
) -> ParserResult<CommandOutput> {
    fn deobfuscate_command(args: &mut Vec<CommandElem>, ps: &mut PowerShellSession) {
        use base64::prelude::*;
        let mut index_to_decode = vec![];
        let mut args = args.iter_mut().map(Some).collect::<Vec<_>>();
        for (i, arg) in args.iter_mut().enumerate() {
            if let Some(CommandElem::Parameter(s)) = arg {
                let p = s.to_ascii_lowercase();
                if let Some(_stripped) = "-encodedcommand".strip_prefix(&p) {
                    index_to_decode.push(i + 1);
                    *s = "-command".to_string();
                }
            }
        }

        for i in index_to_decode {
            if let Some(CommandElem::Argument(Val::ScriptText(s))) = &mut args[i] {
                if let Ok(decoded_bytes) = BASE64_STANDARD.decode(s.clone()) {
                    if let Ok(decoded_str) = String::from_utf16(
                        &decoded_bytes
                            .chunks(2)
                            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                            .collect::<Vec<u16>>(),
                    ) {
                        if let Ok(script_result) = ps.parse_input(&decoded_str) {
                            if script_result.deobfuscated().is_empty() {
                                *s = decoded_str.into();
                            } else {
                                *s = script_result.deobfuscated();
                            }
                        } else {
                            log::warn!("Failed to deobfuscate: {}", &decoded_str);
                            *s = decoded_str.into();
                        }
                    }
                }
            }
        }
    }

    deobfuscate_command(args, ps);

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
        let s = p.parse_input(input).unwrap();
        assert_eq!(
            s.result().to_string(),
            vec!["2", "4", "6", "8", "10"].join(NEWLINE)
        );

        let input = r#"5 | where-object {$_ -eq 5}"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result(), PsValue::Int(5));

        let input = r#"5,4 | where-object {$_ -eq 5}"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result(), PsValue::Int(5));

        let input = r#"5,4 | where {$_ -gt 3}"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(
            s.result(),
            PsValue::Array(vec![PsValue::Int(5), PsValue::Int(4)])
        );

        let input = r#"5,4 | where {$_ -lt 3}"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result(), PsValue::Null);

        let input = r#"@(@{val = 4},@{val = 3}) | where val -lt 4"#;
        let s = p.parse_input(input).unwrap();
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
        let s = p.parse_input(input).unwrap();
        assert_eq!(
            s.result().to_string(),
            vec!["2", "4", "6", "8", "10"].join(NEWLINE)
        );

        let input = r#"5 | % {$_ + 5}"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result(), PsValue::Int(10));

        let input = r#"5,4 | foreach {$_ /2}"#;
        let s = p.parse_input(input).unwrap();
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
        let script_res = p.parse_input(input).unwrap();

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
        let s = p.parse_input(input).unwrap();

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
        let s = p.parse_input(input).unwrap();

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
        let s = p.parse_input(input).unwrap();

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
        let s = p.parse_input(input).unwrap();

        assert_eq!(
            s.deobfuscated().trim(),
            vec![r#"powershell -command Write-Host "tweet, tweet!""#,].join(NEWLINE)
        );
    }

    #[test]
    fn encoded_command2() {
        let mut p = PowerShellSession::new();
        let input = r#"powershell.exe -e JgAgACgAZwBjAG0AIAAoACcAaQBlAHsAMAB9ACcAIAAtAGYAIAAnAHgAJwApACkAIAAoACIAVwByACIAKwAiAGkAdAAiACsAIgBlAC0ASAAiACsAIgBvAHMAdAAgACcASAAiACsAIgBlAGwAIgArACIAbABvACwAIABmAHIAIgArACIAbwBtACAAUAAiACsAIgBvAHcAIgArACIAZQByAFMAIgArACIAaAAiACsAIgBlAGwAbAAhACcAIgApAA=="#;
        let s = p.parse_input(input).unwrap();

        assert_eq!(
            s.deobfuscated().trim(),
            vec![r#"powershell -command gcm iex Write-Host 'Hello, from PowerShell!'"#,]
                .join(NEWLINE)
        );
    }

    #[test]
    fn encoded_command3() {
        let mut p = PowerShellSession::new();
        let input = r#"& (gcm ('ie{0}' -f 'x')) ("Wr"+"it"+"e-H"+"ost 'H"+"el"+"lo, fr"+"om P"+"ow"+"erS"+"h"+"ell!'")"#;
        let s = p.parse_input(input).unwrap();

        assert_eq!(
            s.deobfuscated().trim(),
            vec![r#"gcm iex Write-Host 'Hello, from PowerShell!'"#,].join(NEWLINE)
        );
    }
}
