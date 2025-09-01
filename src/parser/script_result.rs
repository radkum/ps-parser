use std::fmt::Display;

use super::{ParserError, Tokens, Val as InternalVal};
use crate::parser::value::PsString;

#[derive(Debug, Clone, PartialEq)]
pub enum PsValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(u32),
    String(String),
    Array(Vec<PsValue>),
}

impl From<char> for PsValue {
    fn from(c: char) -> Self {
        PsValue::Char(c as u32)
    }
}

impl Display for PsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let val: InternalVal = self.clone().into();
        write!(f, "{}", val)
    }
}

impl From<PsValue> for InternalVal {
    fn from(val: PsValue) -> Self {
        match val {
            PsValue::Null => InternalVal::Null,
            PsValue::Bool(b) => InternalVal::Bool(b),
            PsValue::Int(i) => InternalVal::Int(i),
            PsValue::Float(f) => InternalVal::Float(f),
            PsValue::Char(c) => InternalVal::Char(c),
            PsValue::String(s) => InternalVal::String(PsString(s)),
            PsValue::Array(arr) => {
                InternalVal::Array(arr.iter().map(|v| v.clone().into()).collect())
            }
        }
    }
}

impl From<InternalVal> for PsValue {
    fn from(val: InternalVal) -> Self {
        match val {
            InternalVal::Null => PsValue::Null,
            InternalVal::Bool(b) => PsValue::Bool(b),
            InternalVal::Int(i) => PsValue::Int(i),
            InternalVal::Float(f) => PsValue::Float(f),
            InternalVal::Char(c) => PsValue::Char(c),
            InternalVal::String(PsString(s)) => PsValue::String(s),
            InternalVal::Array(arr) => {
                PsValue::Array(arr.iter().map(|v| v.clone().into()).collect())
            }
            InternalVal::RuntimeObject(obj) => PsValue::String(obj.name()),
        }
    }
}

pub struct ScriptResult {
    output: PsValue,
    deobfuscated: String,
    tokens: Tokens,
    errors: Vec<ParserError>,
}

impl ScriptResult {
    pub(crate) fn new(
        output: InternalVal,
        deobfuscated: String,
        tokens: Tokens,
        errors: Vec<ParserError>,
    ) -> Self {
        Self {
            output: output.into(),
            deobfuscated,
            tokens,
            errors,
        }
    }

    pub fn output(&self) -> PsValue {
        self.output.clone()
    }

    pub fn deobfuscated(&self) -> String {
        self.deobfuscated.clone()
    }

    pub fn tokens(&self) -> Tokens {
        self.tokens.clone()
    }

    pub fn errors(&self) -> Vec<ParserError> {
        self.errors.clone()
    }
}
