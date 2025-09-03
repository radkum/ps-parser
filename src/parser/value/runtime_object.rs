use crate::parser::value::{MethodError, PsString};
use super::TypeInfoTrait;


use super::{MethodResult, Val};
pub type MethodCallType = fn(Val, Vec<Val>) -> MethodResult<Val>;
pub type StaticFnCallType = fn(Vec<Val>) -> MethodResult<Val>;

pub fn get_runtime_object(name: &str) -> MethodResult<Box<dyn RuntimeObject>> {
    match name.to_ascii_lowercase().as_str() {
        "system.convert" => Ok(Box::new(super::system_convert::Convert {})),
        "system.text.encoding" => Ok(Box::new(super::system_encoding::Encoding {})),
        "system.text.encoding::unicode" => Ok(Box::new(super::system_encoding::UnicodeEncoding {})),
        _ => Err(super::MethodError::ObjectNotFound(name.to_string())),
    }
}

pub(crate) trait RuntimeObject: std::fmt::Debug {
    fn get_method(&self, name: &str) -> MethodResult<MethodCallType> {
        Err(MethodError::NotImplemented(name.to_string()))
    }
    fn get_static_fn(&self, name: &str) -> MethodResult<StaticFnCallType> {
        Err(MethodError::NotImplemented(name.to_string()))
    }
    fn get_member(&self, name: &str) -> MethodResult<Val> {
        Err(MethodError::NotImplemented(name.to_string()))
    }
    fn get_static_member(&self, name: &str) -> MethodResult<Val> {
        Err(MethodError::NotImplemented(name.to_string()))
    }
    fn name(&self) -> String {
        format!("{:?}", self)
    }
}

fn get_type(object: Val, _: Vec<Val>) -> MethodResult<Val> {
    Ok(object.type_info()?.into())
}

impl RuntimeObject for Val {
    fn get_method(&self, name: &str) -> MethodResult<MethodCallType> {
        match name {
            "gettype" => return Ok(get_type),
            _ => {},
        }
        match self {
            Val::String(ps) => ps.get_method(name),
            Val::RuntimeObject(s) => s.get_method(name),
            _ => Err(super::MethodError::MethodNotFound(name.to_string())),
        }
    }
    fn get_static_fn(&self, name: &str) -> MethodResult<StaticFnCallType> {
        match self {
            Val::RuntimeObject(runtime_object) => runtime_object.get_static_fn(name),
            _ => Err(super::MethodError::MethodNotFound(name.to_string())),
        }
    }
    fn get_member(&self, name: &str) -> MethodResult<Val> {
        // first check the members
        match self {
            //Val::String(ps) => ps.get_member(name),
            Val::HashTable(ps) => return Ok(ps.get(&name.to_ascii_lowercase()).cloned().unwrap_or_default()),
            _ => {},
        }

        // then check the length property
        if name.eq_ignore_ascii_case("length"){
            return Ok(Val::Int(match self {
                Val::Null => 0,
                Val::String(PsString(s)) => s.len() as i64,
                Val::Array(ar) => ar.len() as i64,
                Val::HashTable(ht) => ht.len() as i64,
                _ => 1,
            }));
        }
        
        Err(super::MethodError::MemberNotFound(name.to_string()))
    }
    fn get_static_member(&self, name: &str) -> MethodResult<Val> {
        match self {
            Val::RuntimeObject(runtime_object) => runtime_object.get_static_member(name),
            _ => Err(super::MethodError::MemberNotFound(name.to_string())),
        }
    }
}
