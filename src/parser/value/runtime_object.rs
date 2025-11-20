use super::{MethodResult, Val, *};
use crate::parser::value::{MethodError, PsString};
pub type MethodCallType = Box<dyn Fn(&Val, Vec<Val>) -> MethodResult<Val>>;
pub type StaticFnCallType = fn(Vec<Val>) -> MethodResult<Val>;

use thiserror_no_std::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RuntimeError {
    #[error("Value \"{0}\" is simple val not runtime, and not defines any type")]
    ValNotDefinesAnyType(String),
    #[error("MethodError: \"{0}\"")]
    Method(MethodError),
    #[error("Member \"{0}\" not found")]
    MemberNotFound(String),
    #[error("MethodError: \"{0}\"")]
    MethodNotFound(String),
    #[error("Index out of bounds: {0}, {1}")]
    IndexOutOfBounds(String, usize),
}

impl From<MethodError> for RuntimeError {
    fn from(value: MethodError) -> Self {
        Self::Method(value)
    }
}

pub type RuntimeResult<T> = core::result::Result<T, RuntimeError>;

pub(crate) trait RuntimeObjectTrait: std::fmt::Debug + std::fmt::Display {
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

    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait>;
}

impl Val {
    fn get_type(&self, _: Vec<Val>) -> MethodResult<Val> {
        Ok(Val::RuntimeType(Box::new(self.ttype())))
    }
}

impl RuntimeObjectTrait for Val {
    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait> {
        Box::new(self.clone())
    }

    fn method(&self, name: &str) -> RuntimeResult<MethodCallType> {
        log::trace!("Val: {self:?} method called with name: {}", name);
        match name {
            "gettype" => return Ok(Box::new(Self::get_type)),
            _ => {}
        }
        match self {
            Val::String(str) => str.method(name),
            Val::RuntimeObject(runtime_object) => runtime_object.method(name),
            //Val::RuntimeType(runtime_type) => runtime_type.method(name),
            _ => Err(super::MethodError::MethodNotFound(name.to_string()).into()),
        }
    }
    fn static_method(&self, name: &str) -> RuntimeResult<StaticFnCallType> {
        match self {
            Val::RuntimeObject(runtime_object) => runtime_object.static_method(name),
            Val::RuntimeType(runtime_type) => runtime_type.static_method(name),
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

        if let Val::RuntimeType(ps) = self {
            return ps.readonly_member(name);
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
            Val::RuntimeType(rt) => rt.readonly_static_member(name),
            _ => Err(RuntimeError::MemberNotFound(name.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue, Variables};

    #[test]
    fn get_type() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());

        let input = r#" $a = ,('m',1234,'s');$a.gettype() "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String(
                "IsPublic\tIsSerial\tName\tBaseType\n--------\t--------\t----\t--------\n    \
                 true\t    true\tObject[]\t   Array"
                    .into()
            )
        );

        let input = r#" $a = ,('m',1234,'s');function Foo($x) { $x[0].GetType().name + $x[2]}; $b = (Foo(1,2,3));$b "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("Int323".into()));

        //this like return "a" + "msi" ".dll", object. However EDR may detect such
        // strings as suspicious, so we test little different string: "assi.dll"
        // let input = r#" $a = ,('m',1234,'s');function Foo($x) { $x[0].GetType().name
        // + $x[2]}; $b = $a.gettype()[0].basetype.name[0] +$a[0][0]
        // +$a[0][2]+(Foo(1,2,3))[0]+([string]$a.gettype())[6]+[char](97+3)
        // +[string][char]((54) | ForEach-Object { $_*2 })*2;$b "#;
        let input = r#" $a = ,('m',1234,'s');function Foo($x) { $x[0].GetType().name + $x[2]}; $b = $a.gettype()[0].basetype.name[0] +$a[0][2] +$a[0][2]+(Foo(1,2,3))[0]+([string]$a.gettype())[6]+[char](97+3) +[string][char]((54) | ForEach-Object { $_*2 })*2;$b "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("AssI.dll".into()));
    }
}
