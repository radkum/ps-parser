use super::{MethodResult, TypeInfoTrait, Val, *};
use crate::parser::value::{MethodError, PsString};
pub type MethodCallType = fn(Val, Vec<Val>) -> MethodResult<Val>;
pub type StaticFnCallType = fn(Vec<Val>) -> MethodResult<Val>;

use thiserror_no_std::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RuntimeError {
    #[error("Value \"{0}\" is simple val not runtime, and not defines any type")]
    ValNotDefinesAnyType(String),
    #[error("MethodErro: \"{0}\"")]
    Method(MethodError),
    #[error("Member \"{0}\" not found")]
    MemberNotFound(String),
    #[error("Index out of bounds: {0}, {1}")]
    IndexOutOfBounds(String, usize),
}

impl From<MethodError> for RuntimeError {
    fn from(value: MethodError) -> Self {
        Self::Method(value)
    }
}

pub type RuntimeResult<T> = core::result::Result<T, RuntimeError>;

pub(crate) trait RuntimeObject: std::fmt::Debug {
    fn method(&self, name: &str) -> RuntimeResult<MethodCallType> {
        Err(MethodError::NotImplemented(name.to_string()).into())
    }
    fn static_method(&self, name: &str) -> RuntimeResult<StaticFnCallType> {
        Err(MethodError::NotImplemented(name.to_string()).into())
    }
    fn member(&mut self, name: &str) -> RuntimeResult<&mut Val> {
        Err(MethodError::NotImplemented(name.to_string()).into())
    }
    fn readonly_member(&self, name: &str) -> RuntimeResult<Val> {
        Err(MethodError::NotImplemented(name.to_string()).into())
    }
    fn readonly_static_member(&self, name: &str) -> RuntimeResult<Val> {
        Err(MethodError::NotImplemented(name.to_string()).into())
    }
    fn name(&self) -> String {
        format!("{:?}", self)
    }
    fn type_definition(&self) -> RuntimeResult<ValType> {
        Err(MethodError::NotImplemented("type_definition()".into()).into())
    }
}

fn get_type(object: Val, _: Vec<Val>) -> MethodResult<Val> {
    Ok(object.type_info()?.into())
}

impl RuntimeObject for Val {
    fn method(&self, name: &str) -> RuntimeResult<MethodCallType> {
        match name {
            "gettype" => return Ok(get_type),
            _ => {}
        }
        match self {
            Val::String(ps) => ps.method(name),
            Val::RuntimeObject(s) => s.method(name),
            _ => Err(super::MethodError::MethodNotFound(name.to_string()).into()),
        }
    }
    fn static_method(&self, name: &str) -> RuntimeResult<StaticFnCallType> {
        match self {
            Val::RuntimeObject(runtime_object) => runtime_object.static_method(name),
            _ => Err(MethodError::MethodNotFound(name.to_string()).into()),
        }
    }

    fn member(&mut self, name: &str) -> RuntimeResult<&mut Val> {
        // first check the members
        if let Val::HashTable(hashtable) = self {
            return hashtable
                .get_mut(&name.to_ascii_lowercase())
                .ok_or_else(|| RuntimeError::MemberNotFound(name.to_string()));
        }

        Err(RuntimeError::MemberNotFound(name.to_string()))
    }

    fn readonly_member(&self, name: &str) -> RuntimeResult<Val> {
        // first check the members
        if let Val::HashTable(ps) = self {
            return Ok(ps
                .get(&name.to_ascii_lowercase())
                .cloned()
                .unwrap_or_default());
        }

        // then check the length property
        if name.eq_ignore_ascii_case("length") {
            return Ok(Val::Int(match self {
                Val::Null => 0,
                Val::String(PsString(s)) => s.len() as i64,
                Val::Array(ar) => ar.len() as i64,
                Val::HashTable(ht) => ht.len() as i64,
                _ => 1,
            }));
        }

        Err(RuntimeError::MemberNotFound(name.to_string()))
    }

    fn readonly_static_member(&self, name: &str) -> RuntimeResult<Val> {
        match self {
            Val::RuntimeObject(runtime_object) => runtime_object.readonly_static_member(name),
            _ => Err(RuntimeError::MemberNotFound(name.to_string())),
        }
    }

    fn type_definition(&self) -> RuntimeResult<ValType> {
        if let Val::RuntimeObject(rt) = self {
            rt.type_definition()
        } else {
            Err(RuntimeError::ValNotDefinesAnyType(self.display()))
        }
    }
}
