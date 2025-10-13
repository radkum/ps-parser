mod scopes;
mod variable;
use std::collections::HashMap;

use phf::phf_map;
pub(super) use scopes::SessionScope;
use thiserror_no_std::Error;
pub(super) use variable::{Scope, VarName};

use crate::parser::Val;
#[derive(Error, Debug, PartialEq, Clone)]
pub enum VariableError {
    #[error("Variable \"{0}\" is not defined")]
    NotDefined(String),
    #[error("Cannot overwrite variable \"{0}\" because it is read-only or constant.")]
    ReadOnly(String),
}

pub type VariableResult<T> = core::result::Result<T, VariableError>;
pub type VariableMap = HashMap<String, Val>;

#[derive(Clone, Default)]
pub struct Variables {
    env: VariableMap,
    global_scope: VariableMap,
    script_scope: VariableMap,
    scope_sessions_stack: Vec<VariableMap>,
    state: State,
    force_var_eval: bool,
    values_persist: bool,
    //special variables
    // status: bool, // $?
    // first_token: Option<String>,
    // last_token: Option<String>,
    // current_pipeline: Option<String>,
}

#[derive(Default, Clone)]
enum State {
    #[default]
    Script,
    Stack(u32),
}

impl Variables {
    const PREDEFINED_VARIABLES: phf::Map<&'static str, Val> = phf_map! {
        "true" => Val::Bool(true),
        "false" => Val::Bool(false),
        "null" => Val::Null,
    };

    pub(crate) fn set_ps_item(&mut self, ps_item: Val) {
        let _ = self.set(
            &VarName::new_with_scope(Scope::Special, "$PSItem".into()),
            ps_item.clone(),
        );
        let _ = self.set(
            &VarName::new_with_scope(Scope::Special, "$_".into()),
            ps_item,
        );
    }

    pub(crate) fn reset_ps_item(&mut self) {
        let _ = self.set(
            &VarName::new_with_scope(Scope::Special, "$PSItem".into()),
            Val::Null,
        );
        let _ = self.set(
            &VarName::new_with_scope(Scope::Special, "$_".into()),
            Val::Null,
        );
    }

    pub fn set_status(&mut self, b: bool) {
        let _ = self.set(
            &VarName::new_with_scope(Scope::Special, "$?".into()),
            Val::Bool(b),
        );
    }

    pub fn status(&mut self) -> bool {
        let Some(Val::Bool(b)) = self.get(&VarName::new_with_scope(Scope::Special, "$?".into()))
        else {
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

    pub fn init(&mut self) {
        if !self.values_persist {
            self.script_scope.clear();
        }
        self.scope_sessions_stack.clear();
        self.state = State::Script;
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
                    "global" => VarName::new_with_scope(Scope::Global, key.to_lowercase()),
                    "script" => VarName::new_with_scope(Scope::Script, key.to_lowercase()),
                    "env" => VarName::new_with_scope(Scope::Env, key.to_lowercase()),
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
                if let Err(err) = self.set(&var_name, parsed_value.clone()) {
                    log::error!("Failed to set variable {:?}: {}", var_name, err);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn script_scope(&self) -> VariableMap {
        self.script_scope.clone()
    }

    pub(crate) fn get_env(&self) -> VariableMap {
        self.env.clone()
    }

    pub(crate) fn get_global(&self) -> VariableMap {
        self.global_scope.clone()
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
        Default::default()
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
        Self {
            force_var_eval: true,
            ..Default::default()
        }
    }

    // not exported in this version
    #[allow(dead_code)]
    pub(crate) fn values_persist(mut self) -> Self {
        self.values_persist = true;
        self
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
        let mut vars = Variables::new();

        // Load all environment variables
        for (key, value) in std::env::vars() {
            // Store environment variables with Env scope so they can be accessed via
            // $env:variable_name
            vars.env
                .insert(key.to_lowercase(), Val::String(value.into()));
        }
        vars
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

    fn const_map_from_scope(&self, scope: &Scope) -> &VariableMap {
        match scope {
            Scope::Global => &self.global_scope,
            Scope::Script => &self.script_scope,
            Scope::Env => &self.env,
            Scope::Local => match self.state {
                State::Script => &self.script_scope,
                State::Stack(depth) => {
                    if depth < self.scope_sessions_stack.len() as u32 {
                        &self.scope_sessions_stack[depth as usize]
                    } else {
                        &self.script_scope
                    }
                }
            },
            Scope::Special => {
                &self.global_scope //todo!(),
            }
        }
    }

    fn local_scope(&mut self) -> &mut VariableMap {
        match self.state {
            State::Script => &mut self.script_scope,
            State::Stack(depth) => {
                if depth < self.scope_sessions_stack.len() as u32 {
                    &mut self.scope_sessions_stack[depth as usize]
                } else {
                    &mut self.script_scope
                }
            }
        }
    }
    fn map_from_scope(&mut self, scope: &Scope) -> &mut VariableMap {
        match scope {
            Scope::Global => &mut self.global_scope,
            Scope::Script => &mut self.script_scope,
            Scope::Env => &mut self.env,
            Scope::Local => self.local_scope(),
            Scope::Special => {
                &mut self.global_scope //todo!(),
            }
        }
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
        let var = self.find_mut_variable_in_scopes(var_name)?;

        if let Some(variable) = var {
            *variable = val;
        } else {
            let map = self.map_from_scope(&var_name.scope.clone().unwrap_or(Scope::Local));
            map.insert(var_name.name.to_ascii_lowercase(), val);
        }

        Ok(())
    }

    pub(crate) fn set_local(&mut self, name: &str, val: Val) -> VariableResult<()> {
        let var_name = VarName::new_with_scope(Scope::Local, name.to_ascii_lowercase());
        self.set(&var_name, val)
    }

    fn find_mut_variable_in_scopes(
        &mut self,
        var_name: &VarName,
    ) -> VariableResult<Option<&mut Val>> {
        let name = var_name.name.to_ascii_lowercase();
        let name_str = name.as_str();

        if let Some(scope) = &var_name.scope {
            let map = self.map_from_scope(scope);
            Ok(map.get_mut(name_str))
        } else {
            if Self::PREDEFINED_VARIABLES.contains_key(name_str) {
                return Err(VariableError::ReadOnly(name.clone()));
            }

            // No scope specified, check local scopes first, then globals
            for local_scope in self.scope_sessions_stack.iter_mut().rev() {
                if local_scope.contains_key(name_str) {
                    return Ok(local_scope.get_mut(name_str));
                }
            }

            if self.script_scope.contains_key(name_str) {
                return Ok(self.script_scope.get_mut(name_str));
            }

            if self.global_scope.contains_key(name_str) {
                return Ok(self.global_scope.get_mut(name_str));
            }

            Ok(None)
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
        let var = self.find_variable_in_scopes(var_name);

        if self.force_var_eval && var.is_none() {
            Some(Val::Null)
        } else {
            var.cloned()
        }
    }

    fn find_variable_in_scopes(&self, var_name: &VarName) -> Option<&Val> {
        let name = var_name.name.to_ascii_lowercase();
        let name_str = name.as_str();

        if let Some(scope) = &var_name.scope {
            let map = self.const_map_from_scope(scope);
            map.get(name_str)
        } else {
            if Self::PREDEFINED_VARIABLES.contains_key(name_str) {
                return Self::PREDEFINED_VARIABLES.get(name_str);
            }

            // No scope specified, check local scopes first, then globals
            for local_scope in self.scope_sessions_stack.iter().rev() {
                if local_scope.contains_key(name_str) {
                    return local_scope.get(name_str);
                }
            }

            if self.script_scope.contains_key(name_str) {
                return self.script_scope.get(name_str);
            }

            if self.global_scope.contains_key(name_str) {
                return self.global_scope.get(name_str);
            }

            None
        }
    }

    pub(crate) fn push_scope_session(&mut self) {
        let current_map = self.local_scope();
        let new_map = current_map.clone();

        self.scope_sessions_stack.push(new_map);
        self.state = State::Stack(self.scope_sessions_stack.len() as u32 - 1);
    }

    pub(crate) fn pop_scope_session(&mut self) {
        match self.scope_sessions_stack.len() {
            0 => todo!(),
            1 => {
                self.scope_sessions_stack.pop();
                self.state = State::Script;
            }
            _ => {
                self.scope_sessions_stack.pop();
                self.state = State::Stack(self.scope_sessions_stack.len() as u32 - 1);
            }
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
    fn test_builtint_objects() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.parse_input(r#" [system.convert]0 "#).unwrap().result(),
            PsValue::Null
        );
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
        let env_variables = p.env_variables();
        assert_eq!(
            env_variables.get("path").unwrap().to_string(),
            std::env::var("PATH").unwrap()
        );
        assert_eq!(
            env_variables.get("tmp").unwrap().to_string(),
            std::env::var("TMP").unwrap()
        );
        assert_eq!(
            env_variables.get("temp").unwrap().to_string(),
            std::env::var("TMP").unwrap()
        );
        assert_eq!(
            env_variables.get("appdata").unwrap().to_string(),
            std::env::var("APPDATA").unwrap()
        );
        assert_eq!(
            env_variables.get("username").unwrap().to_string(),
            std::env::var("USERNAME").unwrap()
        );
        assert_eq!(
            env_variables.get("programfiles").unwrap().to_string(),
            std::env::var("PROGRAMFILES").unwrap()
        );
        assert_eq!(
            env_variables.get("programfiles(x86)").unwrap().to_string(),
            std::env::var("PROGRAMFILES(x86)").unwrap()
        );
    }

    #[test]
    fn test_global_variables() {
        let v = Variables::env();
        let mut p = PowerShellSession::new().with_variables(v);

        p.parse_input(r#" $global:var_int = 5 "#).unwrap();
        p.parse_input(r#" $global:var_string = "global";$script:var_string = "script";$local:var_string = "local" "#).unwrap();

        assert_eq!(
            p.parse_input(r#" $var_int "#).unwrap().result(),
            PsValue::Int(5)
        );
        assert_eq!(
            p.parse_input(r#" $var_string "#).unwrap().result(),
            PsValue::String("global".into())
        );

        let global_variables = p.global_variables();
        assert_eq!(global_variables.get("var_int").unwrap(), &PsValue::Int(5));
        assert_eq!(
            global_variables.get("var_string").unwrap(),
            &PsValue::String("global".into())
        );
    }

    #[test]
    fn test_script_variables() {
        let v = Variables::env();
        let mut p = PowerShellSession::new().with_variables(v);

        let script_res = p
            .parse_input(r#" $script:var_int = 5;$var_string = "assdfa" "#)
            .unwrap();
        let script_variables = script_res.script_variables();
        assert_eq!(script_variables.get("var_int"), Some(&PsValue::Int(5)));
        assert_eq!(
            script_variables.get("var_string"),
            Some(&PsValue::String("assdfa".into()))
        );
    }

    #[test]
    fn test_env_special_cases() {
        let v = Variables::env();
        let mut p = PowerShellSession::new().with_variables(v);
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

        assert_eq!(
            p.safe_eval(r#" ${Env:ProgramFiles(x86):adsf} = 5;${Env:ProgramFiles(x86):adsf} "#)
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

[script]
local_var = "local_value"
        "#;
        let mut variables = Variables::new().values_persist();
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
            p.safe_eval(r#" $script:local_var "#).unwrap().as_str(),
            "\"local_value\""
        );
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

[script]
local_var = "local_value"
        "#;

        let variables = Variables::from_ini_string(input).unwrap().values_persist();
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
            p.safe_eval(r#" $script:local_var "#).unwrap().as_str(),
            "\"local_value\""
        );
        assert_eq!(
            p.safe_eval(r#" $local_var "#).unwrap().as_str(),
            "\"local_value\""
        );
        assert_eq!(
            p.safe_eval(r#" $local:local_var "#).unwrap().as_str(),
            "\"local_value\""
        );
    }
}
