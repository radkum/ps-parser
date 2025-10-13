use std::{collections::HashMap, fmt::Display};

use super::{ParserError, Tokens, Val as InternalVal};
use crate::{
    NEWLINE,
    parser::{StreamMessage, value::PsString},
};

#[derive(Debug, Clone, PartialEq)]
pub enum PsValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(u32),
    String(String),
    Array(Vec<PsValue>),
    HashTable(HashMap<String, PsValue>),
}

impl core::fmt::Display for PsString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PsValue {
    pub fn is_true(&self) -> bool {
        match self {
            PsValue::Bool(b) => *b,
            PsValue::Int(i) => *i != 0,
            PsValue::Float(f) => *f != 0.0,
            PsValue::Char(c) => *c != 0,
            PsValue::String(s) => !s.is_empty(),
            PsValue::Array(arr) => !arr.is_empty(),
            PsValue::HashTable(hash) => !hash.is_empty(),
            PsValue::Null => false,
        }
    }
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
            PsValue::HashTable(hash) => InternalVal::HashTable(
                hash.iter()
                    .map(|(k, v)| (k.clone(), v.clone().into()))
                    .collect(),
            ),
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
            InternalVal::HashTable(hash) => PsValue::HashTable(
                hash.iter()
                    .map(|(k, v)| (k.clone(), v.clone().into()))
                    .collect(),
            ),
            InternalVal::RuntimeObject(obj) => PsValue::String(obj.name()),
            InternalVal::ScriptBlock(sb) => PsValue::String(sb.raw_text),
            InternalVal::ScriptText(st) => PsValue::String(st.clone()),
        }
    }
}

#[derive(Debug)]
pub struct ScriptResult {
    result: PsValue,
    stream: Vec<String>,
    evaluated_statements: Vec<String>,
    tokens: Tokens,
    errors: Vec<ParserError>,
    script_values: HashMap<String, PsValue>,
}

impl ScriptResult {
    pub(crate) fn new(
        result: InternalVal,
        stream: Vec<StreamMessage>,
        evaluated_statements: Vec<String>,
        tokens: Tokens,
        errors: Vec<ParserError>,
        script_values: HashMap<String, PsValue>,
    ) -> Self {
        Self {
            result: result.into(),
            stream: stream
                .iter()
                .cloned()
                .map(|msg| msg.to_string())
                .collect::<Vec<String>>(),
            evaluated_statements,
            tokens,
            errors,
            script_values,
        }
    }

    pub fn result(&self) -> PsValue {
        self.result.clone()
    }

    pub fn deobfuscated_lines(&self) -> Vec<String> {
        self.evaluated_statements.clone()
    }

    pub fn deobfuscated(&self) -> String {
        self.evaluated_statements.join(NEWLINE)
    }

    pub fn tokens(&self) -> Tokens {
        self.tokens.clone()
    }

    pub fn errors(&self) -> Vec<ParserError> {
        self.errors.clone()
    }

    pub fn output(&self) -> String {
        self.stream.join(NEWLINE)
    }

    pub fn output_lines(&self) -> Vec<String> {
        self.stream.clone()
    }

    pub fn script_variables(&self) -> HashMap<String, PsValue> {
        self.script_values.clone()
    }
}
