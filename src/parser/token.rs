use std::fmt::Display;

use super::script_result::PsValue;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    StringExpandable(String, String),
    String(String),
    Expression(String, PsValue),
    Function(String, String, Vec<PsValue>),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Tokens(Vec<Token>);
impl Tokens {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, token: Token) {
        self.0.push(token)
    }

    pub fn all(&self) -> Vec<Token> {
        self.0.clone()
    }

    pub fn strings(&self) -> Vec<Token> {
        self.0
            .iter()
            .filter(|token| matches!(token, Token::String(..)))
            .cloned()
            .collect()
    }

    pub fn expandable_strings(&self) -> Vec<Token> {
        self.0
            .iter()
            .filter(|token| matches!(token, Token::StringExpandable(..)))
            .cloned()
            .collect()
    }

    pub fn expression(&self) -> Vec<Token> {
        self.0
            .iter()
            .filter(|token| matches!(token, Token::Expression(..)))
            .cloned()
            .collect()
    }

    pub fn function(&self) -> Vec<Token> {
        self.0
            .iter()
            .filter(|token| matches!(token, Token::Function(..)))
            .cloned()
            .collect()
    }
}
