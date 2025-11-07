use super::{
    Val,
    params::{Param, Params},
};
use crate::{
    PowerShellSession,
    parser::{CommandElem, CommandOutput, ParserError, ParserResult, Results, ValType},
};
#[derive(Debug, Clone, Default)]
pub(crate) struct ScriptBlock {
    pub params: Params,
    pub body: String,
    pub raw_text: String,
    pub deobfuscated: Vec<String>,
}

impl ScriptBlock {
    pub fn new(params: Vec<Param>, script: String, raw_text: String) -> Self {
        Self {
            params: Params::new(params),
            body: script,
            raw_text,
            deobfuscated: Vec::new(),
        }
    }
    pub fn empty() -> Self {
        Self {
            params: Params::new(Vec::new()),
            body: String::new(),
            raw_text: String::new(),
            deobfuscated: Vec::new(),
        }
    }

    pub fn from_command_elements(command_elements: &[CommandElem]) -> Self {
        let elements = command_elements
            .iter()
            .map(|arg| arg.display())
            .collect::<Vec<_>>()
            .join(" ");

        Self {
            params: Params::new(Vec::new()),
            body: format!("$_.{}", elements),
            raw_text: String::new(),
            deobfuscated: Vec::new(),
        }
    }
    // pub fn to_function(&self, name: &str, scope: &Option<Scope>) -> String {
    //     if let Some(scope) = scope {
    //         format!("function {scope}:{name}(){}", self.deobfuscated_string())
    //     } else {
    //         format!("function {name}(){}", self.deobfuscated_string())
    //     }
    // }
}

impl std::fmt::Display for ScriptBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.raw_text)
    }
}

impl ScriptBlock {
    pub fn deobfuscated_string(&self) -> String {
        let params = self
            .params
            .0
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        format!("{{\n {}; {} \n}}", params, self.deobfuscated.join("\n"))
    }
    pub fn with_params(self, params: Vec<Param>) -> ScriptBlock {
        ScriptBlock {
            params: Params::new(params),
            body: self.body,
            raw_text: self.raw_text,
            deobfuscated: self.deobfuscated,
        }
    }

    pub fn run_mut(
        &mut self,
        command_args: Vec<CommandElem>,
        ps: &mut PowerShellSession,
        ps_item: Option<Val>,
    ) -> ParserResult<CommandOutput> {
        if self.body.is_empty() {
            return Ok(CommandOutput::new(Val::Null, vec![]));
        }
        if let Some(item) = ps_item {
            ps.variables.set_ps_item(item.clone());
        }

        let args = command_args
            .iter()
            .filter_map(|arg| {
                if let CommandElem::Argument(val) = arg {
                    Some(val.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<Val>>();

        for (i, param) in self.params.0.iter().enumerate() {
            let val = args
                .get(i)
                .cloned()
                .unwrap_or(param.default_value().unwrap_or(Val::Null));
            ps.variables
                .set_local(param.name(), val)
                .map_err(ParserError::from)?;
        }

        //first we need match "switch" parameters
        for param in self.params.0.iter() {
            for (i, arg) in command_args.iter().enumerate() {
                if *arg == CommandElem::Parameter(param.command_param()) {
                    //scpecial handle for switch parameters
                    if param.ttype() == Some(ValType::Switch) {
                        ps.variables
                            .set_local(param.name(), Val::Bool(true))
                            .map_err(ParserError::from)?;
                        //args.remove(i);
                    } else {
                        let next_arg =
                            if let Some(CommandElem::Argument(val)) = command_args.get(i + 1) {
                                let v = val.clone();
                                v.cast(&param.ttype().unwrap_or(ValType::String))
                                    .unwrap_or(Val::Null)
                            } else {
                                Val::Null
                            };

                        ps.variables
                            .set_local(param.name(), next_arg)
                            .map_err(ParserError::from)?;
                        //args.remove(i+1);
                        //args.remove(i);
                    }

                    break;
                }
            }
        }

        let (
            script_last_output,
            Results {
                output: _output,
                deobfuscated,
            },
        ) = ps.parse_subscript(self.body.as_str())?;
        //output.into_iter().for_each(|f| ps.add_output_statement(f));
        deobfuscated
            .iter()
            .for_each(|f| self.deobfuscated.push(f.clone()));
        Ok(CommandOutput::new(script_last_output, deobfuscated))
    }

    pub fn run(
        &self,
        args: Vec<CommandElem>,
        ps: &mut PowerShellSession,
        ps_item: Option<Val>,
    ) -> ParserResult<CommandOutput> {
        let mut self_clone = self.clone();
        self_clone.run_mut(args, ps, ps_item)
    }
}

#[cfg(test)]
mod tests {
    use crate::{NEWLINE, PowerShellSession};

    #[test]
    fn test_script_block() {
        let mut p = PowerShellSession::new();
        let input = r#"$elo = 3;$sb = { param($x, $y = 4); $x+$y+$elo};&$sb 1 2"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result().to_string(), "6".to_string());
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$elo = 3", "$sb = {param($x, $y = 4); $x+$y+$elo}", "6",].join(NEWLINE)
        );
        assert_eq!(script_res.output(), "6".to_string());
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn test_script_block_default_args() {
        let mut p = PowerShellSession::new();
        let input = r#"$elo = 3;$sb = { param($x, $y = 4); $x+$y+$elo};.$sb 1"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result().to_string(), "8".to_string());
    }

    #[test]
    fn test_non_existing_script_block() {
        let mut p = PowerShellSession::new();
        let input = r#"$elo = 3;$sb = { param($x, $y = 4); $x+$y+$elo};.$sb2 1"#;
        let script_res = p.parse_input(input).unwrap();
        assert!(script_res.result().to_string().is_empty(),);
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                "$elo = 3",
                "$sb = {param($x, $y = 4); $x+$y+$elo}",
                ".$sb2 1",
            ]
            .join(NEWLINE)
        );
        assert!(script_res.output().is_empty(),);
        assert_eq!(script_res.errors().len(), 1);
        assert_eq!(
            script_res.errors()[0].to_string(),
            "VariableError: Variable \"sb2\" is not defined"
        );
    }

    #[test]
    fn test_script_block_value_assignment() {
        let mut p = PowerShellSession::new();
        let input = r#"$scriptBlock = {param($x, $y) return $x + $y};& $scriptBlock 10 20"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.result().to_string(), "30".to_string());
    }

    #[test]
    fn test_script_block_without_assignment() {
        let mut p = PowerShellSession::new();
        let input = r#"& {param($x, $y) return $x + $y} 10 20 40"#;
        let s = p.parse_input(input).unwrap();
        assert_eq!(s.deobfuscated(), "30".to_string());
        assert_eq!(s.result().to_string(), "30".to_string());
    }
}
