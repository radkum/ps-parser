mod runtime_type;
pub(crate) mod type_info;

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

pub(super) use runtime_type::RuntimeTypeTrait;
use smart_default::SmartDefault;
pub(super) use type_info::ObjectType;
use type_info::{ArrayType, ValueType};

use super::{Convert, RuntimeResult, Val, ValError, ValResult, system_encoding::Encoding};

#[derive(Debug, SmartDefault, PartialEq, Clone)]
pub enum ValType {
    #[default]
    Null,
    Bool,
    Int,
    Float,
    Char,
    String,
    Array(Option<Box<ValType>>),
    HashTable,
    ScriptBlock,
    ScriptText,
    RuntimeObject(String),
    TypeInfo,
    Switch,
}
impl std::fmt::Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            ValType::Null => "".to_string(),
            ValType::Char => "Char".to_string(),
            ValType::Bool => "Boolean".to_string(),
            ValType::Int => "Int32".to_string(),
            ValType::Float => "Double".to_string(),
            ValType::String => "String".to_string(),
            ValType::HashTable => "Hashtable".to_string(),
            ValType::ScriptBlock => "ScriptBlock".to_string(),
            ValType::ScriptText => "ScriptText".to_string(),
            ValType::Array(t) => {
                if let Some(name) = t {
                    format!("{}[]", name)
                } else {
                    "Object[]".to_string()
                }
            }
            ValType::RuntimeObject(rt) => rt.to_string(),
            ValType::TypeInfo => "RuntimeType".to_string(),
            ValType::Switch => "Switch".to_string(),
        };
        write!(f, "{}", x)
    }
}

const CONVERT: Convert = Convert {};
const ENCODING: Encoding = Encoding {};
pub static RUNTIME_TYPE_MAP: LazyLock<Mutex<HashMap<String, Box<dyn RuntimeTypeTrait>>>> =
    LazyLock::new(|| {
        Mutex::new(HashMap::from([
            ("system.convert".into(), Box::new(CONVERT) as _),
            ("system.text.encoding".into(), Box::new(ENCODING) as _),
        ]))
    });
impl ValType {
    pub(crate) fn cast(s: &str) -> ValResult<Self> {
        let mut s = s.to_ascii_lowercase();
        if "object" == s.as_str() || "object[]" == s.as_str() {
            s = "array".into();
        }
        s.retain(|c| !c.is_whitespace());
        if let Some(prefix) = s.strip_suffix("[]") {
            return Ok(Self::Array(Some(Box::new(Self::cast(prefix)?))));
        }

        let t = match s.as_str() {
            "char" | "byte" => Self::Char,
            "bool" => Self::Bool,
            "int" | "long" | "decimal" => Self::Int,
            "float" | "double" => Self::Float,
            "string" => Self::String,
            "array" => Self::Array(None),
            "object" => Self::Array(None),
            "scriptblock" => Self::ScriptBlock,
            "hashtable" => Self::HashTable,
            "switch" => Self::Switch,
            _ => {
                if let Ok(map) = RUNTIME_TYPE_MAP.try_lock()
                    && map.contains_key(s.as_str())
                {
                    return Ok(Self::RuntimeObject(s));
                }
                return Err(ValError::UnknownType(s));
            }
        };
        Ok(t)
    }

    pub(crate) fn runtime_type_from_str(s: &str) -> ValResult<Val> {
        let val_type = Self::cast(s)?;
        val_type.runtime()
    }

    pub(crate) fn runtime(&self) -> ValResult<Val> {
        Ok(Val::RuntimeType(match self {
            ValType::RuntimeObject(name) => {
                let map = RUNTIME_TYPE_MAP
                    .try_lock()
                    .map_err(|_| ValError::UnknownType(name.to_string()))?;
                map.get(name.as_str())
                    .ok_or_else(|| ValError::UnknownType(name.to_string()))?
                    .clone_rt()
            }
            _ => Box::new(self.clone()),
        }))
    }
}

impl RuntimeTypeTrait for ValType {
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        match self {
            ValType::Null => unreachable!(),
            ValType::Char | ValType::Bool | ValType::Switch | ValType::Int | ValType::Float => {
                Box::new(ValueType {})
            }
            ValType::String
            | ValType::HashTable
            | ValType::ScriptText
            | ValType::ScriptBlock
            | ValType::TypeInfo
            | ValType::RuntimeObject(_) => Box::new(ObjectType {}),
            ValType::Array(_) => Box::new(ArrayType {}),
            //ValType::TypeInfo => "System.Reflection.TypeInfo".to_string(),
        }
    }
    fn name(&self) -> String {
        match self {
            ValType::Null => "".to_string(),
            ValType::Char => "Char".to_string(),
            ValType::Bool => "Boolean".to_string(),
            ValType::Int => "Int32".to_string(),
            ValType::Float => "Double".to_string(),
            ValType::String => "String".to_string(),
            ValType::HashTable => "Hashtable".to_string(),
            ValType::ScriptBlock => "ScriptBlock".to_string(),
            ValType::ScriptText => "ScriptText".to_string(),
            ValType::Array(t) => {
                if let Some(name) = t {
                    format!("{}[]", name.name())
                } else {
                    "Object[]".to_string()
                }
            }
            ValType::RuntimeObject(rt) => rt.to_string(),
            ValType::TypeInfo => "RuntimeType".to_string(),
            ValType::Switch => "Switch".to_string(),
        }
    }

    fn type_definition(&self) -> ValType {
        self.clone()
    }

    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}
