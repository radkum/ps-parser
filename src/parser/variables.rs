mod variable;
use std::collections::HashMap;

use thiserror_no_std::Error;
pub(super) use variable::{Scope, VarName, VarProp, Variable};

use crate::parser::Val;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum VariableError {
    #[error("Variable \"{0}\" is not defined")]
    NotDefined(String),
    #[error("Cannot overwrite variable \"{0}\" because it is read-only or constant.")]
    ReadOnly(String),
}

pub type VariableResult<T> = core::result::Result<T, VariableError>;

#[derive(Default, Clone)]
pub struct Variables {
    map: HashMap<VarName, Variable>,
    force_var_eval: bool,
    //special variables
    // status: bool, // $?
    // first_token: Option<String>,
    // last_token: Option<String>,
    // current_pipeline: Option<String>,
}

impl Variables {
    fn const_variables() -> HashMap<VarName, Variable> {
        HashMap::from([
            (
                VarName::new(Scope::Global, "true".to_ascii_lowercase()),
                Variable::new(VarProp::ReadOnly, Val::Bool(true)),
            ),
            (
                VarName::new(Scope::Global, "false".to_ascii_lowercase()),
                Variable::new(VarProp::ReadOnly, Val::Bool(false)),
            ),
            (
                VarName::new(Scope::Global, "null".to_ascii_lowercase()),
                Variable::new(VarProp::ReadOnly, Val::Null),
            ),
        ])
    }

    pub(crate) fn set_ps_item(&mut self, ps_item: Val) {
        let _ = self.set(
            &VarName::new(Scope::Special, "$PSItem".into()),
            ps_item.clone(),
        );
        let _ = self.set(&VarName::new(Scope::Special, "$_".into()), ps_item);
    }

    pub(crate) fn reset_ps_item(&mut self) {
        let _ = self.set(&VarName::new(Scope::Special, "$PSItem".into()), Val::Null);
        let _ = self.set(&VarName::new(Scope::Special, "$_".into()), Val::Null);
    }

    pub fn set_status(&mut self, b: bool) {
        let _ = self.set(&VarName::new(Scope::Special, "$?".into()), Val::Bool(b));
    }

    pub fn status(&mut self) -> bool {
        let Some(Val::Bool(b)) = self.get(&VarName::new(Scope::Special, "$?".into())) else {
            return false;
        };
        b
    }

    pub fn load<R: std::io::Read>(&mut self, reader: R) -> Result<(), Box<dyn std::error::Error>> {
        let mut config_parser = configparser::ini::Ini::new();
        let conf = config_parser.load_from_stream(reader)?;

        for (section_name, properties) in &conf {
            for (key, value) in properties {
                let Some(value) = value else {
                    continue;
                };

                let var_name = match section_name.as_str() {
                    "global" => VarName::new(Scope::Global, key.to_lowercase()),
                    "local" => VarName::new(Scope::Local, key.to_lowercase()),
                    _ => {
                        continue;
                    }
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
                if let Some(variable) = self.map.get(&var_name) {
                    if variable.prop == VarProp::ReadOnly {
                        log::warn!("Skipping read-only variable: {:?}", var_name);
                        continue;
                    }
                }

                self.map
                    .insert(var_name, Variable::new(VarProp::ReadWrite, parsed_value));
            }
        }
        Ok(())
    }

    pub fn env() -> Self {
        let mut map = Self::const_variables();

        // Load all environment variables
        for (key, value) in std::env::vars() {
            // Store environment variables with Env scope so they can be accessed via
            // $env:variable_name
            map.insert(
                VarName::new(Scope::Env, key.to_lowercase()),
                Variable::new(VarProp::ReadWrite, Val::String(value.into())),
            );
        }

        Self {
            map,
            force_var_eval: true,
        }
    }

    pub fn new() -> Self {
        let map = Self::const_variables();

        Self {
            map,
            force_var_eval: false,
        }
    }

    pub fn force_eval() -> Self {
        let map = Self::const_variables();

        Self {
            map,
            force_var_eval: true,
        }
    }

    /// Create a new Variables instance with variables loaded from an INI file
    pub fn from_ini_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut variables = Self::new();
        let mut file = std::fs::File::open(path)?;
        variables.load(&mut file)?;
        Ok(variables)
    }

    /// Create a new Variables instance with variables loaded from an INI file
    pub fn from_ini_string(ini_string: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut variables = Self::new();
        let mut reader = std::io::Cursor::new(ini_string);
        variables.load(&mut reader)?;
        Ok(variables)
    }

    pub(crate) fn get(&self, var_name: &VarName) -> Option<Val> {
        //todo: handle special variables and scopes

        let mut var = self.map.get(var_name).map(|v| v.value.clone());
        if self.force_var_eval && var.is_none() {
            var = Some(Val::Null);
        }

        var
    }

    pub(crate) fn set(&mut self, var_name: &VarName, val: Val) -> VariableResult<()> {
        if let Some(variable) = self.map.get_mut(var_name) {
            if let VarProp::ReadOnly = variable.prop {
                log::error!("You couldn't modify a read-only variable");
                Err(VariableError::ReadOnly(var_name.name.to_string()))
            } else {
                variable.value = val;
                Ok(())
            }
        } else {
            self.map
                .insert(var_name.clone(), Variable::new(VarProp::ReadWrite, val));
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Variables;
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn test_builtin_variables() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" $true "#).unwrap().as_str(), "True");
        assert_eq!(p.safe_eval(r#" $false "#).unwrap().as_str(), "False");
        assert_eq!(p.safe_eval(r#" $null "#).unwrap().as_str(), "");
    }

    #[test]
    fn test_env_variables() {
        let v = Variables::env();
        let mut p = PowerShellSession::new().with_variables(v);
        assert_eq!(
            p.safe_eval(r#" $env:path "#).unwrap().as_str(),
            std::env::var("PATH").unwrap()
        );
        assert_eq!(
            p.safe_eval(r#" $env:username "#).unwrap().as_str(),
            std::env::var("USERNAME").unwrap()
        );
        assert_eq!(
            p.safe_eval(r#" $env:tEMp "#).unwrap().as_str(),
            std::env::var("TEMP").unwrap()
        );
        assert_eq!(
            p.safe_eval(r#" $env:tMp "#).unwrap().as_str(),
            std::env::var("TMP").unwrap()
        );
        assert_eq!(
            p.safe_eval(r#" $env:cOmputername "#).unwrap().as_str(),
            std::env::var("COMPUTERNAME").unwrap()
        );
        assert_eq!(
            p.safe_eval(r#" $env:programfiles "#).unwrap().as_str(),
            std::env::var("PROGRAMFILES").unwrap()
        );
        assert_eq!(
            p.safe_eval(r#" $env:temp "#).unwrap().as_str(),
            std::env::var("TEMP").unwrap()
        );
        assert_eq!(
            p.safe_eval(r#" ${Env:ProgramFiles(x86)} "#)
                .unwrap()
                .as_str(),
            std::env::var("ProgramFiles(x86)").unwrap()
        );

        p.safe_eval(r#" $global:program = $env:programfiles + "\program" "#)
            .unwrap();
        assert_eq!(
            p.safe_eval(r#" $global:program "#).unwrap().as_str(),
            format!("{}\\program", std::env::var("PROGRAMFILES").unwrap())
        );
        assert_eq!(
            p.safe_eval(r#" $program "#).unwrap().as_str(),
            format!("{}\\program", std::env::var("PROGRAMFILES").unwrap())
        );

        p.safe_eval(r#" ${Env:ProgramFiles(x86):adsf} = 5 "#)
            .unwrap();
        assert_eq!(
            p.safe_eval(r#" ${Env:ProgramFiles(x86):adsf} "#)
                .unwrap()
                .as_str(),
            5.to_string()
        );
        assert_eq!(
            p.safe_eval(r#" ${Env:ProgramFiles(x86)} "#)
                .unwrap()
                .as_str(),
            std::env::var("ProgramFiles(x86)").unwrap()
        );
    }

    #[test]
    fn special_last_error() {
        let input = r#"3+"01234 ?";$a=5;$a;$?"#;

        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(input).unwrap().as_str(), "True");

        let input = r#"3+"01234 ?";$?"#;
        assert_eq!(p.safe_eval(input).unwrap().as_str(), "False");
    }

    #[test]
    fn test_from_ini() {
        let input = r#"[global]
name = radek
age = 30
is_admin = true
height = 5.9
empty_value =

[local]
local_var = "local_value"
        "#;
        let mut variables = Variables::new();
        variables.load(input.as_bytes()).unwrap();
        let mut p = PowerShellSession::new().with_variables(variables);

        assert_eq!(
            p.parse_input(r#" $global:name "#).unwrap().result(),
            PsValue::String("radek".into())
        );
        assert_eq!(
            p.parse_input(r#" $global:age "#).unwrap().result(),
            PsValue::Int(30)
        );
        assert_eq!(p.safe_eval(r#" $false "#).unwrap().as_str(), "False");
        assert_eq!(p.safe_eval(r#" $null "#).unwrap().as_str(), "");
        assert_eq!(
            p.safe_eval(r#" $local:local_var "#).unwrap().as_str(),
            "\"local_value\""
        );
    }

    #[test]
    fn test_from_ini_string() {
        let input = r#"[global]
name = radek
age = 30
is_admin = true
height = 5.9
empty_value =

[local]
local_var = "local_value"
        "#;

        let variables = Variables::from_ini_string(input).unwrap();
        let mut p = PowerShellSession::new().with_variables(variables);

        assert_eq!(
            p.parse_input(r#" $global:name "#).unwrap().result(),
            PsValue::String("radek".into())
        );
        assert_eq!(
            p.parse_input(r#" $global:age "#).unwrap().result(),
            PsValue::Int(30)
        );
        assert_eq!(p.safe_eval(r#" $false "#).unwrap().as_str(), "False");
        assert_eq!(p.safe_eval(r#" $null "#).unwrap().as_str(), "");
        assert_eq!(
            p.safe_eval(r#" $local:local_var "#).unwrap().as_str(),
            "\"local_value\""
        );
    }
}
