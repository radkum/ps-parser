use super::script_result::PsValue;

#[derive(Clone)]
pub enum Token {
    StringExpandable(String, String),
    String(String),
    Expression(String, PsValue),
    Function(String, String, Vec<PsValue>),
}

pub type Tokens = Vec<Token>;
