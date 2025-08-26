use crate::parser::value::Val;

pub enum Token {
    StringExpandableToken(String, String),
    StringToken(String),
    Expression(String, Val),
    Function(String, String, Vec<Val>),
}

pub type Tokens = Vec<Token>;
