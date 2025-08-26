use std::collections::HashMap;

use thiserror_no_std::Error;

use super::Val;

#[derive(Error, Debug, PartialEq)]
pub enum VariableError {
    #[error("Variable \"{0}\" is not defined")]
    NotDefined(String),
    #[error("Cannot overwrite variable \"{0}\" because it is read-only or constant.")]
    ReadOnly(String),
}

pub type VariableResult<T> = core::result::Result<T, VariableError>;

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

pub struct Variables {
    map: HashMap<String, (VarProp, Val)>,
    force_var_eval: bool,
}

impl Variables {
    fn const_variables() -> HashMap<String, (VarProp, Val)> {
        HashMap::from([
            (
                "true".to_ascii_lowercase(),
                (VarProp::ReadOnly, Val::Bool(true)),
            ),
            (
                "false".to_ascii_lowercase(),
                (VarProp::ReadOnly, Val::Bool(false)),
            ),
            ("null".to_ascii_lowercase(), (VarProp::ReadOnly, Val::Null)),
        ])
    }

    pub fn load(&mut self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        // Load variables from an INI file
        let Some(str_path) = path.to_str() else {
            return Err("Invalid path".into());
        };
        let conf = ini::ini!(str_path);
        
        for (section_name, properties) in &conf {
            for (key, value) in properties {
                let Some(value) = value else {
                    continue;
                };

                // Create variable name with section prefix if not global
                let var_name = if section_name == "global" {
                    key.to_lowercase()
                } else {
                    format!("{}:{}", section_name.to_lowercase(), key.to_lowercase())
                };
                
                // Try to parse the value as different types
                let parsed_value = if let Ok(bool_val) = value.parse::<bool>() {
                    Val::Bool(bool_val)
                } else if let Ok(int_val) = value.parse::<i64>() {
                    Val::Int(int_val)
                } else if let Ok(float_val) = value.parse::<f64>() {
                    Val::Float(float_val)
                } else if value.is_empty() {
                    Val::Null
                } else {
                    Val::String(value.clone().into())
                };
                
                // Insert the variable (overwrite if it exists and is not read-only)
                if let Some((prop, _)) = self.map.get(&var_name) {
                    match prop {
                        VarProp::ReadOnly => {
                            log::warn!("Skipping read-only variable: {}", var_name);
                            continue;
                        }
                        _ => {}
                    }
                }
                
                self.map.insert(var_name, (VarProp::UserDefined, parsed_value));
            }
        }
        
        Ok(())
    }

    pub(crate) fn env() -> Self {
        let mut map = Self::const_variables();
        
        // Load all environment variables
        for (key, value) in std::env::vars() {
            // Convert environment variable name to PowerShell convention (usually prefixed with $env:)
            let ps_var_name = format!("env:{}", key.to_lowercase());
            map.insert(ps_var_name, (VarProp::Env, Val::String(value.into())));
        }
        
        Self {
            map,
            force_var_eval: true,
        }
    }

    pub(crate) fn new() -> Self {
        let map = Self::const_variables();

        Self {
            map,
            force_var_eval: true,
        }
    }

    /// Create a new Variables instance with variables loaded from an INI file
    pub(crate) fn from_ini(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut variables = Self::new();
        variables.load(path)?;
        Ok(variables)
    }

    /// Create a new Variables instance with both environment variables and INI file variables
    pub(crate) fn env_with_ini(ini_path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut variables = Self::env();
        variables.load(ini_path)?;
        Ok(variables)
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

    pub(crate) fn set(&mut self, name: &str, val: Val) -> VariableResult<()> {
        // if !Self::CONSTANTS_VAR_MAP.contains_key(name) {
        //     self.0.insert(name.to_ascii_lowercase(), val);
        // }
        if let Some((prop, var)) = self.map.get_mut(name.to_ascii_lowercase().as_str()) {
            if let VarProp::ReadOnly = prop {
                log::error!("You couldn't modify a read-only variable");
                Err(VariableError::ReadOnly(name.to_string()))
            } else {
                *var = val;
                Ok(())
            }
        } else {
            self.map
                .insert(name.to_ascii_lowercase(), (VarProp::user_defined(), val));
            Ok(())
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
