use thiserror_no_std::Error;

use super::Val;

#[derive(Error, Debug, Clone)]
pub enum MethodError {
    #[error("Method \"{0}\" not found")]
    MethodNotFound(String),

    #[error("Member \"{0}\" not found")]
    MemberNotFound(String),

    #[error("Method \"{0}\" not implemented")]
    NotImplemented(String),

    #[error("Object \"{0}\" not found")]
    ObjectNotFound(String),

    #[error("Incorrect arguments \"{1:?}\" for method \"{0}\"")]
    IncorrectArgs(String, Vec<String>),

    #[error("RuntimeError: {}", .0.to_string())]
    RuntimeError(String),

    #[error("You cannot call a method \"{0}\" on a null-valued expression.")]
    NullExpression(String),
}
pub type MethodResult<T> = core::result::Result<T, MethodError>;

impl From<Box<dyn std::error::Error + Send + Sync>> for MethodError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        MethodError::RuntimeError(err.to_string())
    }
}

impl PartialEq for MethodError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MethodError::MethodNotFound(a), MethodError::MethodNotFound(b)) => a == b,
            (MethodError::ObjectNotFound(a), MethodError::ObjectNotFound(b)) => a == b,
            (MethodError::IncorrectArgs(a, b), MethodError::IncorrectArgs(c, d)) => {
                a == c && b == d
            }
            (MethodError::RuntimeError(a), MethodError::RuntimeError(b)) => *a == *b,
            _ => false,
        }
    }
}

impl MethodError {
    pub(crate) fn new_incorrect_args(name: &str, args: Vec<Val>) -> Self {
        MethodError::IncorrectArgs(
            name.to_string(),
            args.iter().map(|v| format!("{:?}", v)).collect(),
        )
    }
}
