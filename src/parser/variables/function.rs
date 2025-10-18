use std::collections::HashMap;

use super::{Val, Variables};
use crate::{
    PowerShellSession,
    parser::{CommandOutput, ParserResult, ScriptBlock},
};

pub type FunctionPredType =
    Box<dyn Fn(Vec<Val>, &mut PowerShellSession) -> ParserResult<CommandOutput>>;
pub(super) type FunctionMap = HashMap<String, ScriptBlock>;

impl Variables {
    pub(crate) fn get_function(&mut self, name: &str) -> Option<FunctionPredType> {
        let sb = self.functions.get(name).cloned()?;
        let fun = move |params, ps: &mut crate::PowerShellSession| {
            let sb = sb.clone();
            sb.run(params, ps, None)
        };
        Some(Box::new(fun))
    }
}
