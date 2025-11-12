use std::collections::HashMap;

use thiserror_no_std::Error;

use super::Val;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum TypeError {
    #[error("You cannot call a method \"{0}\" on a null-valued expression.")]
    NullExpression(String),
}
pub type TypeResult<T> = core::result::Result<T, TypeError>;

pub(crate) trait TypeInfoTrait {
    fn type_info(&self) -> TypeResult<TypeInfo>;
}
pub(crate) struct TypeInfo {
    pub is_public: bool,
    pub is_serial: bool,
    pub name: String,
    pub base_type: String,
}

impl TypeInfoTrait for Val {
    fn type_info(&self) -> TypeResult<TypeInfo> {
        if let Val::NonDisplayed(inner) = self {
            return inner.type_info();
        }
        let (is_public, is_serial, base_type) = match self {
            Val::Null => Err(TypeError::NullExpression("GetType".to_string()))?,
            Val::Char(_)
            | Val::Bool(_)
            | Val::Int(_)
            | Val::Float(_)
            | Val::String(_)
            | Val::HashTable(_)
            | Val::ScriptText(_)
            | Val::ScriptBlock(_) => (true, true, "System.Object"),
            Val::Array(_) => (true, true, "System.Array"),
            Val::RuntimeObject(_) => (false, true, "System.Reflection.TypeInfo"),
            _ => panic!("Unreachable"),
        };

        let name = match self {
            Val::Null => Err(TypeError::NullExpression("GetType".to_string()))?,
            Val::Char(_) => "Char",
            Val::Bool(_) => "Boolean",
            Val::Int(_) => "Int32",
            Val::Float(_) => "Double",
            Val::String(_) => "String",
            Val::HashTable(_) => "Hashtable",
            Val::ScriptBlock(_) => "ScriptBlock",
            Val::ScriptText(_) => "ScriptText",
            Val::Array(_) => "Object[]",
            Val::RuntimeObject(_) => "RuntimeType",
            _ => panic!("Unreachable"),
        };

        Ok(TypeInfo {
            is_public,
            is_serial,
            name: name.to_string(),
            base_type: base_type.to_string(),
        })
    }
}

impl From<TypeInfo> for Val {
    fn from(info: TypeInfo) -> Self {
        let mut table = HashMap::new();
        table.insert("IsPublic".to_ascii_lowercase(), Val::Bool(info.is_public));
        table.insert("IsSerial".to_ascii_lowercase(), Val::Bool(info.is_serial));
        table.insert("Name".to_ascii_lowercase(), Val::String(info.name.into()));
        table.insert(
            "BaseType".to_ascii_lowercase(),
            Val::String(info.base_type.into()),
        );
        Val::HashTable(table)
    }
}
