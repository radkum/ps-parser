use super::Val;
use crate::{
    PowerShellSession,
    parser::{CommandOutput, ParserError, ParserResult, Results},
};

#[derive(Debug, Clone)]
pub struct Param {
    name: String,
    //ttype: Option<ValType>,
    default_value: Option<Val>,
}

impl Param {
    pub fn new(name: String, default_value: Option<Val>) -> Self {
        Self {
            name,
            default_value,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    // pub fn ttype(&self) -> Option<&ValType> {
    //     self.ttype.as_ref()
    // }

    pub fn default_value(&self) -> Option<Val> {
        self.default_value.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScriptBlock {
    pub params: Vec<Param>,
    pub body: String,
    pub raw_text: String,
}

impl ScriptBlock {
    pub fn new(params: Vec<Param>, script: String, raw_text: String) -> Self {
        Self {
            params,
            body: script,
            raw_text,
        }
    }
}

impl std::fmt::Display for ScriptBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.raw_text)
    }
}

impl ScriptBlock {
    pub fn with_params(self, params: Vec<Param>) -> ScriptBlock {
        ScriptBlock {
            params,
            body: self.body,
            raw_text: self.raw_text,
        }
    }

    pub fn run(
        &self,
        args: Vec<Val>,
        ps: &mut PowerShellSession,
        ps_item: Option<Val>,
    ) -> ParserResult<CommandOutput> {
        if let Some(item) = ps_item {
            ps.variables.set_ps_item(item.clone());
        }

        for (i, param) in self.params.iter().enumerate() {
            let val = args
                .get(i)
                .cloned()
                .unwrap_or(param.default_value().unwrap_or(Val::Null));
            ps.variables
                .set_local(param.name(), val)
                .map_err(|e| ParserError::from(e))?;
        }

        let (
            script_last_output,
            Results {
                output: _output,
                deobfuscated,
            },
        ) = ps.parse_subscript(self.body.as_str())?;
        Ok(CommandOutput::new(script_last_output, vec![], deobfuscated))
    }
}
