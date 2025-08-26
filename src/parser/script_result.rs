use super::{ParserError, Tokens, Val};

pub struct ScriptResult {
    output: Val,
    deobfuscated: String,
    tokens: Tokens,
    errors: Vec<ParserError>,
}

impl ScriptResult {
    pub fn new(
        output: Val,
        deobfuscated: String,
        tokens: Tokens,
        errors: Vec<ParserError>,
    ) -> Self {
        Self {
            output,
            deobfuscated,
            tokens,
            errors,
        }
    }
}
