use std::collections::HashMap;

use super::Val;

pub(crate) struct Variables(HashMap<String, Val>);

impl Variables {
    pub(crate) fn new() -> Self {
        Self(HashMap::from([
            ("true".to_ascii_lowercase(), Val::Bool(true)),
            ("false".to_ascii_lowercase(), Val::Bool(false)),
        ]))
    }

    
    pub(crate) fn get(&self, name: &str, _scope: Option<String>) -> Val {
        //handle special variables and scopes
        self.0
            .get(name.to_ascii_lowercase().as_str())
            .unwrap_or(&Val::Null)
            .clone()
    }

    pub(crate) fn set(&mut self, name: &str, _scope: Option<String>, val: Val) {
        self.0.insert(name.to_ascii_lowercase(), val);
    }
}
