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
        panic!("{} not implemented", name)
    }
    fn get_static_fn(&self, name: &str) -> MethodResult<StaticFnCallType> {
        panic!("{} not implemented", name)
    }
    fn get_member(&self, name: &str) -> MethodResult<Val> {
        panic!("{} not implemented", name)
    }
    fn get_static_member(&self, name: &str) -> MethodResult<Val> {
        panic!("{} not implemented", name)
    }
    fn name(&self) -> String {
        todo!()
    }
}

impl RuntimeObject for Val {
    fn get_method(&self, name: &str) -> MethodResult<MethodCallType> {
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
        match self {
            Val::String(ps) => ps.get_member(name),
            _ => Err(super::MethodError::MemberNotFound(name.to_string())),
        }
    }
    fn get_static_member(&self, name: &str) -> MethodResult<Val> {
        match self {
            Val::RuntimeObject(runtime_object) => runtime_object.get_static_member(name),
            _ => Err(super::MethodError::MemberNotFound(name.to_string())),
        }
    }
}
