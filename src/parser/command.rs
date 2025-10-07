use std::{collections::HashMap, sync::LazyLock};

use thiserror_no_std::Error;

use super::{SessionScope, StreamMessage, Val, value::ScriptBlock};
use crate::{PowerShellSession, ScriptResult, parser::ParserError};

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub val: Val,                      // Regular return value
    pub deobfuscated: Option<String>,  // Message to a specific stream
    pub stream: Option<StreamMessage>, // Message to a specific stream
}

impl CommandOutput {
    pub fn new(val: Val, streams: Vec<StreamMessage>, deobfuscated: Vec<String>) -> Self {
        Self {
            val,
            deobfuscated: if deobfuscated.is_empty() {
                None
            } else {
                Some(deobfuscated.join(crate::NEWLINE))
            },
            stream: if streams.is_empty() {
                None
            } else {
                let stream_msg = streams
                    .into_iter()
                    .map(|stream| stream.content)
                    .collect::<Vec<_>>()
                    .join(crate::NEWLINE);
                Some(stream_msg.into())
            },
        }
    }
}

impl From<ScriptResult> for CommandOutput {
    fn from(script_result: ScriptResult) -> Self {
        CommandOutput {
            val: script_result.result().into(),
            deobfuscated: script_result.deobfuscated().into(),
            stream: StreamMessage::success(script_result.output()).into(),
        }
    }
}

impl From<Val> for CommandOutput {
    fn from(val: Val) -> Self {
        CommandOutput {
            val,
            deobfuscated: None,
            stream: None,
        }
    }
}

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

type CommandResult<T> = core::result::Result<T, CommandError>;

#[derive(Debug)]
pub enum CommandInner {
    Cmdlet(String),
    //Function(String),
    Path(String),
    ScriptBlock(ScriptBlock),
}

#[derive(Debug)]
pub struct Command {
    command_inner: CommandInner,
    scope: SessionScope,
}

impl Command {
    pub(crate) fn script_block(script_block: ScriptBlock) -> Self {
        Self {
            command_inner: CommandInner::ScriptBlock(script_block),
            scope: SessionScope::Current,
        }
    }

    pub(crate) fn cmdlet(cmdlet: &str) -> Self {
        Self {
            command_inner: CommandInner::Cmdlet(cmdlet.to_string()),
            scope: SessionScope::Current,
        }
    }

    pub(crate) fn path(path: &str) -> Self {
        Self {
            command_inner: CommandInner::Path(path.to_string()),
            scope: SessionScope::Current,
        }
    }

    pub(crate) fn set_session_scope(&mut self, scope: SessionScope) {
        self.scope = scope;
    }
}

pub(crate) type CommandPredType =
    fn(Vec<CommandElem>, Option<&mut PowerShellSession>) -> CommandResult<CommandOutput>;

impl Command {
    const COMMAND_MAP: LazyLock<HashMap<&'static str, CommandPredType>> = LazyLock::new(|| {
        HashMap::from([
            ("write-output", write_output as CommandPredType),
            ("write-warning", write_warning as CommandPredType),
            ("write-host", write_host as CommandPredType),
            ("write-error", write_error as CommandPredType),
            ("write-verbose", write_verbose as CommandPredType),
            ("where-object", where_object as CommandPredType),
        ])
    });

    pub(crate) fn get(name: &str) -> Option<CommandPredType> {
        Self::COMMAND_MAP.get(name).copied()
    }

    pub(crate) fn execute(
        &self,
        ps: &mut PowerShellSession,
        args: Vec<CommandElem>,
    ) -> CommandResult<CommandOutput> {
        let new_scope = matches!(self.scope, SessionScope::New);

        if new_scope {
            ps.push_scope_session();
        }
        let res = match &self.command_inner {
            CommandInner::ScriptBlock(sb) => Ok(ps.eval_script_block(
                sb,
                None,
                args.iter()
                    .filter_map(|x| {
                        if let CommandElem::Argument(v) = x {
                            Some(v.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Val>>(),
            )?),
            CommandInner::Cmdlet(name) => {
                let Some(f) = Self::get(&name.to_ascii_lowercase()) else {
                    return Err(CommandError::NotFound(name.into()));
                };
                f(args, Some(ps))
            }
            CommandInner::Path(path) => Err(CommandError::NotFound(path.into())),
        };

        if new_scope {
            ps.pop_scope_session();
        }
        res
    }
}

#[derive(Debug)]
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
            CommandElem::Argument(v) => v.cast_to_script(),
            CommandElem::ArgList(s) => s.clone(),
        }
    }
}

// Where-Object cmdlet implementation
fn where_object(
    args: Vec<CommandElem>,
    ps: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    log::debug!("args: {:?}", args);
    if args.len() != 2 {
        return Err(CommandError::IncorrectArgs(
            "Where-Object requires exactly two arguments".into(),
        ));
    }

    let CommandElem::Argument(Val::Array(elements)) = &args[0] else {
        return Err(CommandError::IncorrectArgs(
            "First argument must be an array".into(),
        ));
    };

    let CommandElem::Argument(Val::ScriptBlock(script_block)) = &args[1] else {
        return Err(CommandError::IncorrectArgs(
            "Second argument must be a script block".into(),
        ));
    };

    let Some(ps) = ps else {
        return Err(CommandError::IncorrectArgs("Where-Object".into()));
    };

    let filtered_elements = elements
        .iter()
        .filter(|&element| {
            match ps.eval_script_block(script_block, Some(element.clone()), vec![]) {
                Err(er) => {
                    ps.errors.push(er);
                    false
                }
                Ok(b) => b.val.cast_to_bool(),
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    Ok(CommandOutput {
        val: Val::Array(filtered_elements),
        deobfuscated: None,
        stream: None,
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
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);
    let deobfuscated = format!(
        "Write-Host {}",
        args.iter()
            .map(|p| p.display())
            .collect::<Vec<_>>()
            .join(" ")
    );

    Ok(CommandOutput {
        val: Val::Null,
        deobfuscated: Some(deobfuscated),
        stream: Some(StreamMessage::success(message)),
    })
}
// Write-Output cmdlet implementation
fn write_output(
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);
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
        stream: None,
    })
}

// Write-Warning cmdlet implementation (mimics PowerShell's Write-Warning)
fn write_warning(
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);
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
        stream: None,
    })
}

// Write-Error cmdlet implementation
fn write_error(
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);
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
        stream: None,
    })
}

// Write-Verbose cmdlet implementation
fn write_verbose(
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);
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
        stream: None,
    })
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
                format!(
                    "Write-Output \"{}\"",
                    std::env::var("PROGRAMFILES").unwrap()
                )
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_res.output(), std::env::var("PROGRAMFILES").unwrap());
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn test_script_block() {
        let mut p = PowerShellSession::new();
        let input = r#"$elo = 3;$sb = { param($x, $y = 4); $x+$y+$elo};&$sb 1 2"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result().to_string(), "6".to_string());
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$elo = 3", "$sb = {param($x, $y = 4); $x+$y+$elo}",].join(NEWLINE)
        );
        assert_eq!(script_res.output(), "6".to_string());
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn test_script_block_default_args() {
        let mut p = PowerShellSession::new();
        let input = r#"$elo = 3;$sb = { param($x, $y = 4); $x+$y+$elo};.$sb 1"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result().to_string(), "8".to_string());
    }

    #[test]
    fn test_non_existing_script_block() {
        let mut p = PowerShellSession::new();
        let input = r#"$elo = 3;$sb = { param($x, $y = 4); $x+$y+$elo};.$sb2 1"#;
        let script_res = p.parse_input(input).unwrap();
        assert!(script_res.result().to_string().is_empty(),);
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                "$elo = 3",
                "$sb = {param($x, $y = 4); $x+$y+$elo}",
                ".$sb2 1",
            ]
            .join(NEWLINE)
        );
        assert!(script_res.output().is_empty(),);
        assert_eq!(script_res.errors().len(), 1);
        assert_eq!(
            script_res.errors()[0].to_string(),
            "VariableError: Variable \"sb2\" is not defined"
        );
    }

    #[test]
    fn test_script_block_value_assignment() {
        let mut p = PowerShellSession::new();
        let input = r#"$scriptBlock = {param($x, $y) return $x + $y};& $scriptBlock 10 20"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result().to_string(), "30".to_string());
    }

    #[test]
    fn test_script_block_without_assignment() {
        let mut p = PowerShellSession::new();
        let input = r#"& {param($x, $y) return $x + $y} 10 20 40"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result().to_string(), "30".to_string());
    }
}
