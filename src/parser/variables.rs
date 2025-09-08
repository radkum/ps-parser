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

    pub fn load_from_file(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut config_parser = configparser::ini::Ini::new();
        let map = config_parser.load(path)?;
        self.load(map)
    }

    pub fn load_from_string(&mut self, ini_string: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut config_parser = configparser::ini::Ini::new();
        let map = config_parser.read(ini_string.into())?;
        self.load(map)
    }

    fn load(
        &mut self,
        conf_map: HashMap<String, HashMap<String, Option<String>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (section_name, properties) in conf_map {
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

    /// Creates a new empty Variables container.
    ///
    /// # Arguments
    ///
    /// * initializes the container with PowerShell built-in variables like
    ///   `$true`, `$false`, `$null`, and `$?`. If `false`,
    ///
    /// # Returns
    ///
    /// A new `Variables` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ps_parser::Variables;
    ///
    /// // Create with built-in variables
    /// let vars_with_builtins = Variables::new();
    ///
    /// // Create empty
    /// let empty_vars = Variables::new();
    /// ```
    pub fn new() -> Variables {
        let map = Self::const_variables();

        Self {
            map,
            force_var_eval: false,
        }
    }

    /// Creates a new Variables container with forced evaluation enabled.
    ///
    /// This constructor creates a Variables instance that will return
    /// `Val::Null` for undefined variables instead of returning `None`.
    /// This is useful for PowerShell script evaluation where undefined
    /// variables should be treated as `$null` rather than causing errors.
    ///
    /// # Returns
    ///
    /// A new `Variables` instance with forced evaluation enabled and built-in
    /// variables initialized.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ps_parser::{Variables, PowerShellSession};
    ///
    /// // Create with forced evaluation
    /// let vars = Variables::force_eval();
    /// let mut session = PowerShellSession::new().with_variables(vars);
    ///
    /// // Undefined variables will evaluate to $null instead of causing errors
    /// let result = session.safe_eval("$undefined_variable").unwrap();
    /// assert_eq!(result, "");  // $null displays as empty string
    /// ```
    ///
    /// # Behavior Difference
    ///
    /// - `Variables::new()`: Returns `None` for undefined variables
    /// - `Variables::force_eval()`: Returns `Val::Null` for undefined variables
    ///
    /// This is particularly useful when parsing PowerShell scripts that may
    /// reference variables that haven't been explicitly defined, allowing
    /// the script to continue execution rather than failing.
    pub fn force_eval() -> Self {
        let map = Self::const_variables();

        Self {
            map,
            force_var_eval: true,
        }
    }

    /// Loads all environment variables into a Variables container.
    ///
    /// This method reads all environment variables from the system and stores
    /// them in the `env` scope, making them accessible as
    /// `$env:VARIABLE_NAME` in PowerShell scripts.
    ///
    /// # Returns
    ///
    /// A new `Variables` instance containing all environment variables.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ps_parser::{Variables, PowerShellSession};
    ///
    /// let env_vars = Variables::env();
    /// let mut session = PowerShellSession::new().with_variables(env_vars);
    ///
    /// // Access environment variables
    /// let path = session.safe_eval("$env:PATH").unwrap();
    /// let username = session.safe_eval("$env:USERNAME").unwrap();
    /// ```
    pub fn env() -> Variables {
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

    /// Loads variables from an INI configuration file.
    ///
    /// This method parses an INI file and loads its key-value pairs as
    /// PowerShell variables. Variables are organized by INI sections, with
    /// the `[global]` section creating global variables and other sections
    /// creating scoped variables.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the path of the INI file to load.
    ///
    /// # Returns
    ///
    /// * `Result<Variables, VariableError>` - A Variables instance with the
    ///   loaded data, or an error if the file cannot be read or parsed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ps_parser::{Variables, PowerShellSession};
    /// use std::path::Path;
    ///
    /// // Load from INI file
    /// let variables = Variables::from_ini_string("[global]\nname = John Doe\n[local]\nlocal_var = \"local_value\"").unwrap();
    /// let mut session = PowerShellSession::new().with_variables(variables);
    ///
    /// // Access loaded variables
    /// let name = session.safe_eval("$global:name").unwrap();
    /// let local_var = session.safe_eval("$local:local_var").unwrap();
    /// ```
    ///
    /// # INI Format
    ///
    /// ```ini
    /// # Global variables (accessible as $global:key)
    /// [global]
    /// name = John Doe
    /// version = 1.0
    ///
    /// # Local scope variables (accessible as $local:key)
    /// [local]
    /// temp_dir = /tmp
    /// debug = true
    /// ```
    pub fn from_ini_string(ini_string: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut variables = Self::new();
        variables.load_from_string(ini_string)?;
        Ok(variables)
    }

    /// Create a new Variables instance with variables loaded from an INI file
    pub fn from_ini_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut variables = Self::new();
        variables.load_from_file(path)?;
        Ok(variables)
    }

    /// Sets the value of a variable in the specified scope.
    ///
    /// # Arguments
    ///
    /// * `var_name` - The variable name and scope information.
    /// * `val` - The value to assign to the variable.
    ///
    /// # Returns
    ///
    /// * `Result<(), VariableError>` - Success or an error if the variable is
    ///   read-only.
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

    /// Retrieves the value of a variable from the appropriate scope.
    ///
    /// # Arguments
    ///
    /// * `var_name` - The variable name and scope information.
    ///
    /// # Returns
    ///
    /// * `VariableResult<Val>` - The variable's value, or an error if not
    ///   found.
    pub(crate) fn get(&self, var_name: &VarName) -> Option<Val> {
        //todo: handle special variables and scopes

        let mut var = self.map.get(var_name).map(|v| v.value.clone());
        if self.force_var_eval && var.is_none() {
            var = Some(Val::Null);
        }

        var
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
        variables.load_from_string(input).unwrap();
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
