#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub(crate) struct VarName {
    pub scope: Option<Scope>,
    pub name: String,
}

impl VarName {
    pub(crate) fn new(scope: Option<Scope>, name: String) -> Self {
        Self { scope, name }
    }

    pub(crate) fn new_with_scope(scope: Scope, name: String) -> Self {
        Self {
            scope: Some(scope),
            name,
        }
    }
}

impl std::fmt::Display for VarName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(scope) = &self.scope {
            match scope {
                Scope::Global => write!(f, "$global:{}", self.name),
                Scope::Local => write!(f, "$local:{}", self.name),
                Scope::Env => write!(f, "$env:{}", self.name),
                Scope::Special => write!(f, "{}", self.name),
                Scope::Script => write!(f, "$script:{}", self.name),
            }
        } else {
            write!(f, "${}", self.name)
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub(crate) enum Scope {
    Special,
    Global,
    Script,
    Local,
    Env,
}

impl From<&str> for Scope {
    fn from(s: &str) -> Self {
        match s {
            "env" => Scope::Env,
            "global" => Scope::Global,
            "local" => Scope::Local,
            "script" => Scope::Script,
            _ => Scope::Global,
        }
    }
}
