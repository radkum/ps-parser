use std::num::ParseFloatError;

use thiserror_no_std::Error;

use super::{
    CommandError, PestError,
    predicates::OpError,
    value::{MethodError, ValError},
    variables::VariableError,
};

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ParserError {
    #[error("PestError: {0}")]
    PestError(String),

    #[error("ValError: {0}")]
    ValError(ValError),

    #[error("VariableError: {0}")]
    VariableError(VariableError),

    #[error("OperatorError: {0}")]
    OpError(OpError),

    #[error("MethodError: {0}")]
    MethodError(MethodError),

    #[error("CommandError: {0}")]
    CommandError(CommandError),

    #[error("ParseFloatError: {0}")]
    ParseFloatError(ParseFloatError),

    #[error("NotImplementedError: {0}")]
    NotImplemented(String),
}

impl From<PestError> for ParserError {
    fn from(value: PestError) -> Self {
        Self::PestError(value.to_string())
    }
}

impl From<ValError> for ParserError {
    fn from(value: ValError) -> Self {
        Self::ValError(value)
    }
}

impl From<VariableError> for ParserError {
    fn from(value: VariableError) -> Self {
        Self::VariableError(value)
    }
}

impl From<OpError> for ParserError {
    fn from(value: OpError) -> Self {
        Self::OpError(value)
    }
}

impl From<MethodError> for ParserError {
    fn from(value: MethodError) -> Self {
        Self::MethodError(value)
    }
}

impl From<CommandError> for ParserError {
    fn from(value: CommandError) -> Self {
        Self::CommandError(value)
    }
}

impl std::error::Error for ParserError {}

impl From<ParseFloatError> for ParserError {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}
