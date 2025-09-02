use crate::parser::Val;

#[derive(Debug, PartialEq)]
pub(crate) struct Variable {
    pub value: Val,
    pub prop: VarProp,
}

impl Variable {
    pub(crate) fn new(prop: VarProp, value: Val) -> Self {
        Self { value, prop }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub(crate) struct VarName {
    pub scope: Scope,
    pub name: String,
}

impl VarName {
    pub(crate) fn new(scope: Scope, name: String) -> Self {
        Self { scope, name }
    }
}

impl std::fmt::Display for VarName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.scope {
            Scope::Global => write!(f, "${}", self.name),
            Scope::Local => write!(f, "$local:{}", self.name),
            Scope::Env => write!(f, "$env:{}", self.name),
            Scope::Special => write!(f, "{}", self.name),
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub(crate) enum Scope {
    Special,
    Global,
    Local,
    Env,
}

impl From<&str> for Scope {
    fn from(s: &str) -> Self {
        match s {
            "env" => Scope::Env,
            "global" => Scope::Global,
            "local" => Scope::Local,
            _ => Scope::Global,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum VarProp {
    ReadOnly,
    ReadWrite,
}
