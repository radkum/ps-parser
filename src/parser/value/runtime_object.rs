use super::{MethodResult, Val, *};
use crate::parser::value::{MethodError, PsString};
pub type MethodCallType = Box<dyn FnMut(&mut Val, Vec<Val>) -> MethodResult<Val>>;
pub type StaticFnCallType = Box<dyn FnMut(Vec<Val>) -> MethodResult<Val>>;
use thiserror_no_std::Error;

use super::val_type::type_info::RuntimeType;

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
    fn method(&self, method_name: MethodName) -> RuntimeResult<MethodCallType> {
        Err(MethodError::NotImplemented(method_name.name().to_string()).into())
    }
    fn member(&mut self, name: &str) -> RuntimeResult<&mut Val> {
        Err(MethodError::NotImplemented(name.to_string()).into())
    }
    fn readonly_member(&mut self, name: &str) -> RuntimeResult<Val> {
        self.member(name).map(|v| v.clone())
    }
    fn name(&self) -> String {
        format!("{:?}", self)
    }

    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait>;

    fn type_definition(&self) -> Box<dyn RuntimeTypeTrait>;
}

impl Val {
    fn get_type(&mut self, _: Vec<Val>) -> MethodResult<Val> {
        Ok(Val::RuntimeType(self.type_definition()))
    }
}

impl RuntimeObjectTrait for Val {
    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait> {
        Box::new(self.clone())
    }

    fn method(&self, method_name: MethodName) -> RuntimeResult<MethodCallType> {
        let name = method_name.name();
        log::trace!("Val: {self:?} method called with name: {}", name);
        match name {
            "gettype" => return Ok(Box::new(Self::get_type)),
            _ => {}
        }
        match self {
            Val::String(str) => str.method(method_name),
            Val::RuntimeObject(runtime_object) => runtime_object.method(method_name),
            //Val::RuntimeType(runtime_type) => runtime_type.method(name),
            _ => Err(super::MethodError::MethodNotFound(name.to_string()).into()),
        }
    }

    fn member(&mut self, name: &str) -> RuntimeResult<&mut Val> {
        // first check the members
        match self {
            Val::HashTable(hashtable) => hashtable
                .get_mut(&name.to_ascii_lowercase())
                .ok_or_else(|| RuntimeError::MemberNotFound(name.to_string())),
            Val::RuntimeObject(ps) => ps.member(name),
            _ => Err(RuntimeError::MemberNotFound(name.to_string())),
        }
    }

    fn readonly_member(&mut self, name: &str) -> RuntimeResult<Val> {
        // first check the members
        match self {
            Val::HashTable(ps) => {
                return Ok(ps
                    .get(&name.to_ascii_lowercase())
                    .cloned()
                    .unwrap_or_default());
            }
            Val::RuntimeType(ps) => return ps.readonly_member(name),
            Val::RuntimeObject(ps) => return ps.readonly_member(name),
            _ => {}
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

    fn name(&self) -> String {
        match self {
            Val::RuntimeObject(rt) => rt.name(),
            Val::RuntimeType(rt) => rt.name(),
            _ => format!("{:?}", self),
        }
    }

    fn type_definition(&self) -> Box<dyn RuntimeTypeTrait> {
        match self {
            Val::RuntimeType(_) => Box::new(RuntimeType {}),
            Val::RuntimeObject(ro) => ro.type_definition(),
            _ => Box::new(self.ttype()),
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
