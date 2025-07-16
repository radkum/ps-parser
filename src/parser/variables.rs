use std::collections::HashMap;

use super::Val;

pub(crate) struct Variables(HashMap<String, Val>);

impl Variables {
    pub(crate) fn new() -> Self {
        Self(HashMap::from([("$null".to_string(), Val::Null)]))
    }

    pub(crate) fn get(&self, name: &str) -> Val {
        self.0.get(name).unwrap_or(&Val::Null).clone()
    }

    pub(crate) fn set(&mut self, name: &str, val: Val) {
        self.0.insert(name.to_string(), val);
    }
}
