use std::num::{ParseFloatError, ParseIntError};

use thiserror_no_std::Error;
#[derive(Error, Debug, PartialEq, Clone)]
pub enum ValError {
    #[error(
        "Specified argument was out of the range of valid values. Operation \"{0}\", Argument: {1}"
    )]
    ArgumentOutOfRange(String, i64),

    #[error("Cannot convert value \"{0}\" to type \"{1}\"")]
    InvalidCast(String, String),

    #[error("Unknown type \"{0}\"")]
    UnknownType(String),

    #[error("Operation \"{0}\" is not defined for types \"{1}\" op \"{2}\"")]
    OperationNotDefined(String, String, String),

    #[error("Can't divide by zero")]
    DividingByZero,

    #[error("Cannot index into a null array")]
    IndexedNullArray,
}

impl From<ParseFloatError> for ValError {
    fn from(_value: ParseFloatError) -> Self {
        Self::InvalidCast("String".to_string(), "Float".to_string())
    }
}
impl From<ParseIntError> for ValError {
    fn from(_value: ParseIntError) -> Self {
        Self::InvalidCast("String".to_string(), "Int".to_string())
    }
}
