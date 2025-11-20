use super::val_type::RuntimeTypeTrait;
use crate::parser::{Val, ValType};

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    name: String,
    ttype: Option<ValType>,
    default_value: Option<Val>,
}

impl Param {
    pub fn new(ttype: Option<ValType>, name: String, default_value: Option<Val>) -> Self {
        Self {
            name,
            ttype,
            default_value,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn command_param(&self) -> String {
        format!("-{}", self.name)
    }

    pub fn ttype(&self) -> Option<ValType> {
        self.ttype.clone()
    }

    pub fn default_value(&self) -> Option<Val> {
        self.default_value.clone()
    }
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ttype = if let Some(ttype) = &self.ttype {
            format!("[{}] ", ttype.name())
        } else {
            "".to_string()
        };

        let default = if let Some(default) = &self.default_value {
            format!(" = {}", default)
        } else {
            "".to_string()
        };

        write!(f, "{ttype}${}{default}", self.name)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Params(pub Vec<Param>);

impl Params {
    pub fn new(params: Vec<Param>) -> Self {
        Self(params)
    }
}

impl std::fmt::Display for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let params_str = self
            .0
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "param (\n{}\n)", params_str)
    }
}
