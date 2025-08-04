use thiserror_no_std::Error;

use super::{PestError, predicates::OpError, value::ValError, variables::VariableError};

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("PestError: {0}")]
    PestError(PestError),

    #[error("ValError: {0}")]
    ValError(ValError),

    #[error("VariableError: {0}")]
    VariableError(VariableError),

    #[error("OperatorError: {0}")]
    OpError(OpError),
}

impl From<PestError> for ParserError {
    fn from(value: PestError) -> Self {
        Self::PestError(value)
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

impl std::error::Error for ParserError {}
