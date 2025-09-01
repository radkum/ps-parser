use std::{collections::HashMap, sync::LazyLock};

use thiserror_no_std::Error;

use super::{StreamMessage, Val};

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub val: Option<Val>,                      // Regular return value
    pub stream_message: Option<StreamMessage>, // Message to a specific stream
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum CommandError {
    #[error("{0} not found")]
    NotFound(String),
}

type CommandResult<T> = core::result::Result<T, CommandError>;

pub(crate) struct Command {}

pub(crate) type CommandPredType = fn(Vec<CommandElem>) -> CommandResult<CommandOutput>;

impl Command {
    const COMMAND_MAP: LazyLock<HashMap<&'static str, CommandPredType>> = LazyLock::new(|| {
        HashMap::from([
            ("write-output", write_output as CommandPredType),
            ("write-warning", write_warning as CommandPredType),
            ("write-host", write_host as CommandPredType),
            ("write-error", write_error as CommandPredType),
            ("write-verbose", write_verbose as CommandPredType),
        ])
    });

    pub(crate) fn get(name: &str) -> Option<CommandPredType> {
        Self::COMMAND_MAP.get(name).copied()
    }

    pub(crate) fn invoke(name: &str, args: Vec<CommandElem>) -> CommandResult<CommandOutput> {
        let Some(f) = Self::get(&name.to_ascii_lowercase()) else {
            return Err(CommandError::NotFound(name.into()));
        };
        f(args)
    }
}
pub(crate) enum CommandElem {
    Parameter(String),
    Argument(Val),
    ArgList(String),
}

// Helper function to extract message from command arguments
fn extract_message(args: &[CommandElem]) -> String {
    args.iter()
        .filter_map(|arg| match arg {
            CommandElem::Parameter(s) | CommandElem::ArgList(s) => Some(s.clone()),
            CommandElem::Argument(val) => Some(val.cast_to_string()),
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// Write-Output cmdlet implementation
fn write_output(args: Vec<CommandElem>) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: Some(Val::String(message.clone().into())),
        stream_message: Some(StreamMessage::success(message)),
    })
}

// Write-Warning cmdlet implementation (mimics PowerShell's Write-Warning)
fn write_warning(args: Vec<CommandElem>) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: Some(Val::String(message.clone().into())),
        stream_message: Some(StreamMessage::warning(message)),
    })
}

// Write-Error cmdlet implementation
fn write_error(args: Vec<CommandElem>) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: Some(Val::String(message.clone().into())),
        stream_message: Some(StreamMessage::error(message)),
    })
}

// Write-Verbose cmdlet implementation
fn write_verbose(args: Vec<CommandElem>) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: Some(Val::String(message.clone().into())),
        stream_message: Some(StreamMessage::verbose(message)),
    })
}

// Write-Host cmdlet implementation (goes directly to console, not capturable)
fn write_host(args: Vec<CommandElem>) -> CommandResult<CommandOutput> {
    let message = extract_message(&args);

    Ok(CommandOutput {
        val: None,
        stream_message: Some(StreamMessage::success(message)),
    })
}
