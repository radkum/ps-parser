use std::{collections::HashMap, sync::LazyLock};

use thiserror_no_std::Error;

use super::{StreamMessage, Val};
use crate::PowerShellSession;

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub val: Option<Val>,                      // Regular return value
    pub stream_message: Option<StreamMessage>, // Message to a specific stream
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum CommandError {
    #[error("{0} not found")]
    NotFound(String),
    #[error("Incorrect arguments for method \"{0}\"")]
    IncorrectArgs(String),
}

type CommandResult<T> = core::result::Result<T, CommandError>;

pub(crate) struct Command {}

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
        ps: &mut PowerShellSession,
        name: &str,
        args: Vec<CommandElem>,
    ) -> CommandResult<CommandOutput> {
        let Some(f) = Self::get(&name.to_ascii_lowercase()) else {
            return Err(CommandError::NotFound(name.into()));
        };
        f(args, Some(ps))
    }
}

#[derive(Debug)]
pub(crate) enum CommandElem {
    Parameter(String),
    Argument(Val),
    #[allow(dead_code)]
    ArgList(String),
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

    let stream_message = StreamMessage::error("".into());
    let filtered_elements = elements
        .into_iter()
        .filter(
            |element| match ps.eval_script_block(script_block, element) {
                Err(er) => {
                    ps.errors.push(er);
                    false
                }
                Ok(b) => b,
            },
        )
        .map(|element| element.clone())
        .collect::<Vec<Val>>();

    if !stream_message.is_empty() {
        Ok(CommandOutput {
            val: Some(Val::Array(filtered_elements)),
            stream_message: Some(stream_message),
        })
    } else {
        Ok(CommandOutput {
            val: Some(Val::Array(filtered_elements)),
            stream_message: None,
        })
    }
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
            CommandElem::Parameter(s) => match s.to_ascii_lowercase().as_str() {
                "-foregroundcolor" => skip = 1,
                _ => {}
            },
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

    Ok(CommandOutput {
        val: None,
        stream_message: Some(StreamMessage::success(message)),
    })
}
// Write-Output cmdlet implementation
fn write_output(
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: Some(Val::String(message.clone().into())),
        stream_message: Some(StreamMessage::success(message)),
    })
}

// Write-Warning cmdlet implementation (mimics PowerShell's Write-Warning)
fn write_warning(
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: Some(Val::String(message.clone().into())),
        stream_message: Some(StreamMessage::warning(message)),
    })
}

// Write-Error cmdlet implementation
fn write_error(
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: Some(Val::String(message.clone().into())),
        stream_message: Some(StreamMessage::error(message)),
    })
}

// Write-Verbose cmdlet implementation
fn write_verbose(
    args: Vec<CommandElem>,
    _: Option<&mut PowerShellSession>,
) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: Some(Val::String(message.clone().into())),
        stream_message: Some(StreamMessage::verbose(message)),
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
        println!("s: {:?}", s);
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
            std::env::var("PROGRAMFILES").unwrap()
        );
        assert_eq!(script_res.errors().len(), 0);
    }
}
