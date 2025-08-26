use std::collections::HashMap;

use thiserror_no_std::Error;

use super::Val;

#[derive(Error, Debug, PartialEq)]
pub enum VariableError {
    #[error("Variable \"{0}\" is not defined")]
    NotDefined(String),
}

enum VarProp {
    ReadOnly,
    UserDefined,
    Env,
}

impl VarProp {
    pub(crate) fn read_only() -> Self {
        Self::ReadOnly
    }

    pub(crate) fn user_defined() -> Self {
        Self::UserDefined
    }
}

pub(crate) struct Variables {
    map: HashMap<String, (VarProp, Val)>,
    force_var_eval: bool,
}

impl Variables {
    // const CONSTANTS_VAR_MAP: LazyLock<HashMap<String, (VarProp, Val)>> =
    //     LazyLock::new(|| {
    //         HashMap::from([
    //             ("true".to_ascii_lowercase(), (VarProp::ReadOnly,
    // Val::Bool(true))),             ("false".to_ascii_lowercase(),
    // (VarProp::ReadOnly, Val::Bool(false))),
    // ("null".to_ascii_lowercase(), (VarProp::ReadOnly, Val::Null)),         ])
    //     });

    pub(crate) fn new(force_var_eval: bool) -> Self {
        let map = HashMap::from([
            (
                "true".to_ascii_lowercase(),
                (VarProp::ReadOnly, Val::Bool(true)),
            ),
            (
                "false".to_ascii_lowercase(),
                (VarProp::ReadOnly, Val::Bool(false)),
            ),
            ("null".to_ascii_lowercase(), (VarProp::ReadOnly, Val::Null)),
        ]);

        Self {
            map,
            force_var_eval,
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<Val> {
        //todo: handle special variables and scopes

        let mut var = self
            .map
            .get(name.to_ascii_lowercase().as_str())
            .map(|v| v.1.clone());

        if self.force_var_eval {
            if var.is_none() {
                var = Some(Val::Null);
            }
        }

        var
    }

    pub(crate) fn set(&mut self, name: &str, val: Val) {
        // if !Self::CONSTANTS_VAR_MAP.contains_key(name) {
        //     self.0.insert(name.to_ascii_lowercase(), val);
        // }
        if let Some((prop, var)) = self.map.get_mut(name.to_ascii_lowercase().as_str()) {
            if let VarProp::ReadOnly = prop {
                //todo
            } else {
                *var = val;
            }
        } else {
            self.map
                .insert(name.to_ascii_lowercase(), (VarProp::user_defined(), val));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::PowerShellSession;

    #[test]
    fn test_variables() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" $true "#).unwrap().as_str(), "True");
        assert_eq!(p.safe_eval(r#" $false "#).unwrap().as_str(), "False");
        assert_eq!(p.safe_eval(r#" $null "#).unwrap().as_str(), "");
    }
}
