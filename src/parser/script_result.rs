use std::fmt::Display;

use super::{ParserError, Tokens, Val as InternalVal};
use crate::parser::{StreamMessage, value::PsString};

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

#[derive(Debug)]
pub struct ScriptResult {
    result: PsValue,
    stream: String,
    deobfuscated: String,
    tokens: Tokens,
    errors: Vec<ParserError>,
}

impl ScriptResult {
    pub(crate) fn new(
        result: InternalVal,
        stream: Vec<StreamMessage>,
        deobfuscated: String,
        tokens: Tokens,
        errors: Vec<ParserError>,
    ) -> Self {
        Self {
            result: result.into(),
            stream: stream
                .iter()
                .cloned()
                .map(|msg| msg.to_string())
                .collect::<Vec<String>>()
                .join("\n"),
            deobfuscated,
            tokens,
            errors,
        }
    }

    pub fn result(&self) -> PsValue {
        self.result.clone()
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

    pub fn stream(&self) -> String {
        self.stream.clone()
    }
}
