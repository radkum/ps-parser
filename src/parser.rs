mod command;
mod error;
mod predicates;
mod script_result;
mod stream_message;
mod token;
mod value;
mod variables;

use std::collections::HashMap;

pub(crate) use command::CommandError;
use command::{Command, CommandElem};
pub(crate) use stream_message::StreamMessage;
use value::{RuntimeObject, ScriptBlock, ValResult};
use variables::Scope;
type ParserResult<T> = core::result::Result<T, ParserError>;
use error::ParserError;
type PestError = pest::error::Error<Rule>;
use pest::Parser;
use pest_derive::Parser;
use predicates::{ArithmeticPred, BitwisePred, LogicalPred, StringPred};
pub use script_result::{PsValue, ScriptResult};
use token::{Token, Tokens};
pub(crate) use value::{Val, ValType};
pub use variables::Variables;
use variables::{VarName, VariableError};

use crate::parser::command::CommandOutput;

type Pair<'i> = ::pest::iterators::Pair<'i, Rule>;
type Pairs<'i> = ::pest::iterators::Pairs<'i, Rule>;

pub(crate) const NEWLINE: &str = "\r\n";

macro_rules! check_rule {
    ($pair:expr, $rule:pat) => {
        if !matches!($pair.as_rule(), $rule) {
            panic!("rule: {:?}", $pair.as_rule());
        }
    };
}

#[derive(Parser)]
#[grammar = "powershell.pest"]
pub struct PowerShellSession {
    variables: Variables,
    tokens: Tokens,
    errors: Vec<ParserError>,
    stream: Vec<StreamMessage>,
}

impl Default for PowerShellSession {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PowerShellSession {
    pub fn new() -> Self {
        Self {
            variables: Variables::new(),
            tokens: Vec::new(),
            errors: Vec::new(),
            stream: Vec::new(),
        }
    }

    pub fn with_variables(mut self, variables: Variables) -> Self {
        self.variables = variables;
        self
    }

    // pub fn errors(self) -> Vec<ParserError> {
    //     self.errors
    // }

    pub fn parse_input(&mut self, input: &str) -> ParserResult<ScriptResult> {
        let mut pairs = PowerShellSession::parse(Rule::program, input)?;
        let program_token = pairs.next().expect("");

        let mut script_statements = Vec::new();
        let mut script_last_output = Val::default();

        if let Rule::program = program_token.as_rule() {
            let pairs = program_token.into_inner();

            for token in pairs {
                let token_str = token.as_str();
                let result = match token.as_rule() {
                    Rule::pipeline_statement => self.eval_pipeline_statement(token.clone()),
                    Rule::EOI => {
                        break;
                    }
                    Rule::pipeline => {
                        //first assign to output, later create from it script line
                        self.eval_pipeline(token.clone())
                    }
                    Rule::statement_terminator => continue,
                    _ => {
                        log::error!("safe_eval not implemented: {:?}", token.as_rule());
                        Ok(Val::String(token_str.into()))
                    }
                };

                self.variables.set_status(result.is_ok());

                script_last_output = match result {
                    Ok(val) => {
                        if val != Val::Null {
                            script_statements.push(val.display());
                        }

                        val
                    }
                    Err(e) => {
                        self.errors.push(e);
                        script_statements.push(token_str.into());
                        Val::Null
                    }
                };
            }
        }

        let deobfuscated = script_statements.join(NEWLINE);

        Ok(ScriptResult::new(
            script_last_output,
            std::mem::take(&mut self.stream),
            deobfuscated,
            std::mem::take(&mut self.tokens),
            std::mem::take(&mut self.errors),
        ))
    }

    pub(crate) fn eval_script_block(
        &mut self,
        input: &ScriptBlock,
        ps_item: &Val,
    ) -> ParserResult<bool> {
        self.variables.set_ps_item(ps_item.clone());
        let res = self.parse_input(input.0.as_str())?.result().is_true();
        self.variables.reset_ps_item();
        Ok(res)
    }

    pub fn safe_eval(&mut self, input: &str) -> ParserResult<String> {
        Ok(self.parse_input(input)?.result().to_string())
    }

    fn eval_statement(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        Ok(match token.as_rule() {
            Rule::pipeline_statement => self.safe_eval_pipeline_statement(token)?,
            Rule::pipeline => self.safe_eval_pipeline(token)?,
            _ => {
                panic!("eval statements not implemented: {:?}", token.as_rule());
            }
        })
    }

    fn eval_statements(&mut self, token: Pair<'a>) -> ParserResult<Vec<Val>> {
        //check_rule!(token, Rule::statements);
        let pairs = token.into_inner();
        let mut statements = vec![];

        for token in pairs {
            let s = self.eval_statement(token)?;
            statements.push(s);
        }
        Ok(statements)
    }

    fn parse_dq(&mut self, token: Pair<'a>) -> ParserResult<String> {
        let mut res_str = String::new();
        let pairs = token.into_inner();
        for token in pairs {
            let token = token.into_inner().next().unwrap();
            let s = match token.as_rule() {
                Rule::variable => self.get_variable(token)?.cast_to_string(),
                Rule::sub_expression => Val::Array(self.eval_statements(token)?).cast_to_string(),
                Rule::backtick_escape => token
                    .as_str()
                    .strip_prefix("`")
                    .unwrap_or_default()
                    .to_string(),
                _ => token.as_str().to_string(),
            };
            res_str.push_str(s.as_str());
        }
        Ok(res_str)
    }

    fn eval_string_literal(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::string_literal);
        let mut pair = token.into_inner();
        let token = pair.next().unwrap();
        let cloned_token = token.clone();

        let mut is_expandable = false;
        let res = match token.as_rule() {
            Rule::doublequoted_string_literal | Rule::doublequoted_multiline_string_literal => {
                is_expandable = true;
                self.parse_dq(token)?
            }
            Rule::singlequoted_string_literal => {
                if let Some(stripped_prefix) = token.as_str().to_string().strip_prefix("'") {
                    if let Some(stripped_suffix) = stripped_prefix.to_string().strip_suffix("'") {
                        stripped_suffix.to_string()
                    } else {
                        panic!("no suffix")
                    }
                } else {
                    panic!("no prefix")
                }
            }
            Rule::singlequoted_multiline_string_literal => {
                let mut res_str = String::new();
                let pairs = token.into_inner();
                for token in pairs {
                    res_str.push_str(token.as_str());
                }
                res_str
            }
            _ => {
                panic!("eval_string_literal - token.rule(): {:?}", token.as_rule());
            }
        };
        let ps_token = if is_expandable {
            Token::StringExpandable(cloned_token.as_str().to_string(), res.clone())
        } else {
            Token::String(cloned_token.as_str().to_string())
        };
        self.tokens.push(ps_token);

        Ok(Val::String(res.into()))
    }

    fn get_variable(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::variable);
        let var_name = Self::parse_variable(token)?;
        let Some(var) = self.variables.get(&var_name) else {
            return Err(ParserError::VariableError(VariableError::NotDefined(
                var_name.name,
            )));
        };
        Ok(var)
    }

    fn parse_variable(token: Pair<'a>) -> ParserResult<VarName> {
        check_rule!(token, Rule::variable);
        let mut pair = token.into_inner();
        let token = pair.next().unwrap();

        Ok(match token.as_rule() {
            Rule::special_variable => VarName::new(Scope::Special, token.as_str().to_string()),
            Rule::parenthesized_variable => {
                Self::parse_variable(token.into_inner().next().unwrap())?
            }
            Rule::braced_variable => {
                let token = token.into_inner().next().unwrap();
                let var = token.as_str().to_ascii_lowercase();
                let splits: Vec<&str> = var.split(":").collect();
                if splits.len() == 2 {
                    VarName::new(Scope::from(splits[0]), splits[1].to_string())
                } else {
                    VarName::new(Scope::Global, var)
                }
            }
            Rule::scoped_variable => {
                let mut pairs = token.into_inner();
                let mut token = pairs.next().unwrap();

                let scope = if token.as_rule() == Rule::scope_keyword {
                    let scope = token.as_str().to_ascii_lowercase();
                    token = pairs.next().unwrap();
                    check_rule!(token, Rule::var_name);
                    Scope::from(scope.as_str())
                } else {
                    Scope::Global
                };
                VarName::new(scope, token.as_str().to_ascii_lowercase())
            }
            _ => {
                panic!("token.rule(): {:?}", token.as_rule());
            }
        })
    }

    fn eval_expression_with_unary_operator(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::expression_with_unary_operator);
        let mut pair = token.into_inner();
        let token = pair.next().unwrap();

        let res = match token.as_rule() {
            Rule::pre_inc_expression => {
                let variable_token = token.into_inner().next().unwrap();
                let var_name = Self::parse_variable(variable_token)?;
                let mut var = self.variables.get(&var_name).unwrap_or_default();
                var.inc()?;

                self.variables.set(&var_name, var.clone())?;
                var
            }
            Rule::pre_dec_expression => {
                let variable_token = token.into_inner().next().unwrap();
                let var_name = Self::parse_variable(variable_token)?;
                let mut var = self.variables.get(&var_name).unwrap_or_default();
                var.dec()?;

                self.variables.set(&var_name, var.clone())?;
                var
            }
            Rule::cast_expression => self.eval_cast_expression(token)?,
            Rule::negate_op => {
                let unary_token = pair.next().unwrap();
                let unary = self.eval_unary_exp(unary_token)?;
                Val::Bool(!unary.cast_to_bool())
            }
            Rule::bitwise_negate_op => {
                let unary_token = pair.next().unwrap();
                let unary = self.eval_unary_exp(unary_token)?;
                Val::Int(!unary.cast_to_int()?)
            }
            _ => {
                panic!("token.rule(): {:?}", token.as_rule());
            }
        };

        Ok(res)
    }

    fn eval_argument_list(&mut self, token: Pair<'a>) -> ParserResult<Vec<Val>> {
        check_rule!(token, Rule::argument_list);
        let pairs = token.into_inner();

        let mut args = Vec::new();
        for token in pairs {
            args.push(self.eval_expression(token)?);
        }

        Ok(args)
    }

    fn eval_member_access(&mut self, token: Pair<'a>) -> ParserResult<String> {
        //check_rule!(token, Rule::member_access);
        let member_name_token = token.into_inner().next().unwrap();
        let member_name = member_name_token.as_str().to_ascii_lowercase();

        Ok(member_name)
    }

    fn method_is_static(&mut self, token: Pair<'a>) -> bool {
        check_rule!(token, Rule::method_invocation);
        let mut pairs = token.into_inner();

        let access = pairs.next().unwrap();
        match access.as_rule() {
            Rule::member_access => false,
            Rule::static_access => true,
            _ => todo!(),
        }
    }

    fn eval_method_invokation(&mut self, token: Pair<'a>) -> ParserResult<(String, Vec<Val>)> {
        check_rule!(token, Rule::method_invocation);
        let token_string = token.as_str().to_string();

        let mut pairs = token.into_inner();

        let access = pairs.next().unwrap();
        //check_rule!(member_access, Rule::member_access);
        let method_name = self.eval_member_access(access)?;

        let args = if let Some(token) = pairs.next() {
            check_rule!(token, Rule::argument_list);
            self.eval_argument_list(token)?
        } else {
            Vec::new()
        };

        self.tokens.push(Token::Function(
            token_string,
            method_name.clone(),
            args.clone().iter().map(|arg| arg.clone().into()).collect(),
        ));
        Ok((method_name, args))
    }

    fn eval_access(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        fn get_member_name(token: Pair<'_>) -> &'_ str {
            token.into_inner().next().unwrap().as_str()
        }
        check_rule!(token, Rule::access);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        let mut object = self.eval_value(token)?;

        for token in pairs {
            match token.as_rule() {
                Rule::static_access => {
                    object = object.get_static_member(get_member_name(token))?;
                }
                Rule::member_access => {
                    object = object.get_member(get_member_name(token))?;
                }
                Rule::method_invocation => {
                    let static_method = self.method_is_static(token.clone());
                    let (function_name, args) = self.eval_method_invokation(token)?;
                    log::trace!("Method: {:?} {:?}", &function_name, &args);
                    object = if static_method {
                        let call = object.get_static_fn(function_name.as_str())?;
                        call(args)?
                    } else {
                        let call = object.get_method(function_name.as_str())?;
                        call(object, args)?
                    };
                }
                Rule::element_access => {
                    let mut pairs = token.into_inner();
                    let index_token = pairs.next().unwrap();
                    check_rule!(index_token, Rule::expression);
                    let index = self.eval_expression(index_token)?;
                    object = object.get_index(index)?;
                }
                _ => {
                    panic!("token.rule(): {:?}", token.as_rule());
                }
            }
        }
        log::debug!("Success eval_access: {:?}", object);
        Ok(object)
    }

    fn parse_access(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::access);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        let mut object = token.as_str().to_string();

        for token in pairs {
            match token.as_rule() {
                Rule::static_access => {
                    object.push_str("::");
                    object.push_str(token.as_str());
                }
                Rule::member_access => {
                    //object.push('.');
                    object.push_str(token.as_str());
                }
                Rule::method_invocation => {
                    let static_method = self.method_is_static(token.clone());
                    let (method_name, args) = self.eval_method_invokation(token)?;

                    let separator = if static_method { "::" } else { "." };
                    object = format!(
                        "{}{separator}{}({:?})",
                        object,
                        method_name.to_ascii_lowercase(),
                        args
                    )
                }
                Rule::element_access => {
                    let mut pairs = token.into_inner();
                    let index_token = pairs.next().unwrap();
                    check_rule!(index_token, Rule::expression);
                    let index = self.eval_expression(index_token)?;
                    object = format!("{}[{}]", object, index);
                }
                _ => {
                    panic!("parse_access token.rule(): {:?}", token.as_rule());
                }
            }
        }
        Ok(Val::String(object.into()))
    }

    fn eval_primary_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::primary_expression);
        let mut pair = token.into_inner();
        let token = pair.next().unwrap();
        let res = match token.as_rule() {
            Rule::access => match self.eval_access(token.clone()) {
                Ok(res) => res,
                Err(err) => {
                    log::info!("eval_access error: {:?}", err);
                    self.errors.push(err);
                    self.parse_access(token)?
                }
            },
            Rule::value => self.eval_value(token)?,
            Rule::post_inc_expression => {
                let variable_token = token.into_inner().next().unwrap();
                let var_name = Self::parse_variable(variable_token)?;
                let mut var = self.variables.get(&var_name).unwrap_or_default();
                let var_to_return = var.clone();

                var.inc()?;
                self.variables.set(&var_name, var.clone())?;

                //if var_to_return.ttype() ==
                var_to_return
            }
            Rule::post_dec_expression => {
                let variable_token = token.into_inner().next().unwrap();
                let var_name = Self::parse_variable(variable_token)?;
                let mut var = self.variables.get(&var_name).unwrap_or_default();
                let var_to_return = var.clone();

                var.dec()?;
                self.variables.set(&var_name, var.clone())?;

                var_to_return
            }
            _ => {
                panic!(
                    "eval_primary_expression: rule: {:?} str: {}",
                    token.as_rule(),
                    token.as_str()
                );
            }
        };

        Ok(res)
    }

    fn eval_type_literal(&mut self, token: Pair<'a>) -> ParserResult<ValType> {
        check_rule!(token, Rule::type_literal);

        let token = token.into_inner().next().unwrap();
        check_rule!(token, Rule::type_spec);
        Ok(ValType::cast(token.as_str())?)
    }

    fn parse_script_block_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::script_block_expression);

        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        Ok(Val::ScriptBlock(ScriptBlock(token.as_str().to_string())))
    }

    fn eval_hash_key(&mut self, token: Pair<'a>) -> ParserResult<String> {
        check_rule!(token, Rule::key_expression);
        let mut pairs = token.into_inner();
        let key_token = pairs.next().unwrap();

        Ok(match key_token.as_rule() {
            Rule::simple_name => key_token.as_str().to_ascii_lowercase(),
            Rule::unary_exp => self
                .eval_unary_exp(key_token)?
                .cast_to_string()
                .to_ascii_lowercase(),
            _ => {
                panic!("key_token.rule(): {:?}", key_token.as_rule());
            }
        })
    }

    fn eval_hash_entry(&mut self, token: Pair<'a>) -> ParserResult<(String, Val)> {
        check_rule!(token, Rule::hash_entry);

        let mut pairs = token.into_inner();
        let token_key = pairs.next().unwrap();
        let token_value = pairs.next().unwrap();

        Ok((
            self.eval_hash_key(token_key)?,
            self.eval_statement(token_value)?,
        ))
    }

    fn eval_hash_literal(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::hash_literal_expression);
        let pairs = token.into_inner();
        let mut hash = HashMap::new();
        for token in pairs {
            let (key, value) = self.eval_hash_entry(token)?;
            hash.insert(key, value);
        }
        Ok(Val::HashTable(hash))
    }

    fn eval_value(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::value);
        let mut pair = token.into_inner();
        let token = pair.next().unwrap();

        let res = match token.as_rule() {
            Rule::parenthesized_expression => {
                let token = token.into_inner().next().unwrap();
                self.safe_eval_pipeline(token)?
            }
            Rule::sub_expression | Rule::array_expression => {
                let statements = self.eval_statements(token)?;
                if statements.len() == 1 && statements[0].ttype() == ValType::Array {
                    statements[0].clone()
                } else {
                    Val::Array(statements)
                }
            }
            Rule::script_block_expression => self.parse_script_block_expression(token)?,
            Rule::hash_literal_expression => self.eval_hash_literal(token)?,
            Rule::string_literal => self.eval_string_literal(token)?,
            Rule::number_literal => self.eval_number_literal(token)?,
            Rule::type_literal => Val::init(self.eval_type_literal(token)?)?,
            Rule::variable => self.get_variable(token)?,
            _ => {
                panic!("token.rule(): {:?}", token.as_rule());
            }
        };
        log::debug!("eval_value - res: {:?}", res);
        Ok(res)
    }

    fn eval_number_literal(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::number_literal);
        let mut negate = false;
        let mut pairs = token.into_inner();
        let mut token = pairs.next().unwrap();

        //first handle prefix sign: + or -
        if token.as_rule() == Rule::minus {
            negate = true;
            token = pairs.next().unwrap();
        } else if token.as_rule() == Rule::plus {
            token = pairs.next().unwrap();
        }

        let mut val = self.eval_number(token)?;

        if negate {
            val.neg()?;
        }

        if let Some(unit) = pairs.next() {
            let unit = unit.as_str().to_ascii_lowercase();
            let unit_int = match unit.as_str() {
                "k" => 1024,
                "m" => 1024 * 1024,
                "g" => 1024 * 1024 * 1024,
                "t" => 1024 * 1024 * 1024 * 1024,
                "p" => 1024 * 1024 * 1024 * 1024 * 1024,
                _ => 1,
            };
            val.mul(Val::Int(unit_int))?;
        }
        Ok(val)
    }

    fn eval_number(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::number);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();
        let v = match token.as_rule() {
            Rule::decimal_integer => {
                let int_val = token.into_inner().next().unwrap();
                Val::Int(int_val.as_str().parse::<i64>().unwrap())
            }
            Rule::hex_integer => {
                let int_val = token.into_inner().next().unwrap();
                Val::Int(i64::from_str_radix(int_val.as_str(), 16).unwrap())
            }
            //todo: parse float in proper way
            Rule::float => {
                let float_str = token.as_str().trim();

                match float_str.parse::<f64>() {
                    Ok(float_val) => Val::Float(float_val),
                    Err(err) => {
                        println!("eval_number - invalid float: {}: asd {}", float_str, err);
                        panic!("eval_number - invalid float: {}: {}", float_str, err);
                    }
                }
            }
            _ => {
                panic!("eval_number - token.rule(): {:?}", token.as_rule());
            }
        };
        Ok(v)
    }

    fn eval_unary_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::unary_exp);
        let token = token.into_inner().next().unwrap();
        match token.as_rule() {
            Rule::expression_with_unary_operator => self.eval_expression_with_unary_operator(token),
            Rule::primary_expression => self.eval_primary_expression(token),
            _ => {
                panic!("eval_unary_exp token.rule(): {:?}", token.as_rule());
            }
        }
    }

    fn eval_array_literal_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::array_literal_exp);
        let mut arr = Vec::new();
        let mut pairs = token.into_inner();
        arr.push(self.eval_unary_exp(pairs.next().unwrap())?);
        for token in pairs {
            arr.push(self.eval_unary_exp(token)?);
        }

        Ok(if arr.len() == 1 {
            arr[0].clone()
        } else {
            Val::Array(arr)
        })
    }

    fn eval_range_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        fn range(left: i64, right: i64) -> Vec<Val> {
            if left < right {
                (left..=right).collect::<Vec<i64>>()
            } else {
                let mut v = (right..=left).collect::<Vec<i64>>();
                v.reverse();
                v
            }
            .into_iter()
            .map(Val::Int)
            .collect::<Vec<Val>>()
        }
        check_rule!(token, Rule::range_exp);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();
        let res = match token.as_rule() {
            Rule::decimal_integer => {
                let int_val = token.into_inner().next().unwrap();
                let left = int_val.as_str().parse::<i64>().unwrap();
                let token = pairs.next().unwrap();
                let right = self.eval_array_literal_exp(token)?.cast_to_int()?;
                Val::Array(range(left, right))
            }
            Rule::array_literal_exp => {
                let res = self.eval_array_literal_exp(token)?;
                if let Some(token) = pairs.next() {
                    let left = res.cast_to_int()?;
                    let right = self.eval_array_literal_exp(token)?.cast_to_int()?;
                    Val::Array(range(left, right))
                } else {
                    res
                }
            }
            _ => {
                panic!("eval_range_exp not implemented: {:?}", token.as_rule());
            }
        };

        Ok(res)
    }

    fn eval_format_impl(&mut self, format: Val, mut pairs: Pairs<'a>) -> ParserResult<Val> {
        fn format_with_vec(fmt: &str, args: Vec<Val>) -> ParserResult<String> {
            fn strange_special_case(fmt: &str, n: i64) -> String {
                fn split_digits(n: i64) -> Vec<u8> {
                    n.abs() // ignore sign for digit splitting
                        .to_string()
                        .chars()
                        .filter_map(|c| c.to_digit(10).map(|opt| opt as u8))
                        .collect()
                }

                //"{0:31sdfg,0100a0b00}" -f 578 evals to 310100a5b78
                let mut digits = split_digits(n);
                digits.reverse();
                let mut fmt_vec = fmt.as_bytes().to_vec();
                fmt_vec.reverse();

                let mut i = 0;
                for digit in digits {
                    while i < fmt_vec.len() {
                        if fmt_vec[i] != b'0' {
                            i += 1
                        } else {
                            fmt_vec[i] = digit + b'0';
                            break;
                        }
                    }
                }
                fmt_vec.reverse();
                String::from_utf8(fmt_vec).unwrap_or_default()
            }

            let mut output = String::new();
            let mut i = 0;

            while i < fmt.len() {
                if fmt[i..].starts_with('{') {
                    if let Some(end) = fmt[i..].find('}') {
                        let token = &fmt[i + 1..i + end];
                        let formatted = if token.contains(':') {
                            let mut parts = token.split(':');
                            let index: usize = if let Some(p) = parts.next() {
                                p.parse().unwrap_or(0)
                            } else {
                                0
                            };

                            let spec = parts.next();
                            match args.get(index) {
                                Some(val) => match spec {
                                    Some(s) if s.starts_with('N') => {
                                        let precision = s[1..].parse::<usize>().unwrap_or(2);
                                        if let Ok(f) = val.cast_to_float() {
                                            format!("{:.1$}", f, precision)
                                        } else {
                                            val.cast_to_string().to_string()
                                        }
                                    }
                                    Some(s) => strange_special_case(s, val.cast_to_int()?),
                                    None => val.cast_to_string().to_string(),
                                },
                                None => format!("{{{}}}", token), /* leave as-is if index out of
                                                                   * bounds */
                            }
                        } else if token.contains(',') {
                            let mut parts = token.split(',');
                            let index: usize = parts.next().unwrap().parse().unwrap_or(0);
                            let spec = parts.next();
                            match args.get(index) {
                                Some(val) => match spec {
                                    Some(s) => {
                                        let spaces = s.parse::<usize>().unwrap_or(0);
                                        let spaces_str = " ".repeat(spaces);
                                        format!("{spaces_str}{}", val.cast_to_string())
                                    }
                                    _ => val.cast_to_string().to_string(),
                                },
                                None => format!("{{{}}}", token), /* leave as-is if index out of
                                                                   * bounds */
                            }
                        } else {
                            let index: usize =
                                Val::String(token.to_string().into()).cast_to_int()? as usize;
                            match args.get(index) {
                                Some(val) => val.cast_to_string().to_string(),
                                None => format!("{{{}}}", token), /* leave as-is if index out of
                                                                   * bounds */
                            }
                        };

                        output.push_str(&formatted);
                        i += end + 1;
                    } else {
                        output.push('{');
                        i += 1;
                    }
                } else {
                    output.push(fmt[i..].chars().next().unwrap());
                    i += 1;
                }
            }

            Ok(output)
        }

        Ok(if let Some(token) = pairs.next() {
            let first_fmt = format.cast_to_string();

            let second_fmt = self.eval_range_exp(token)?;
            let res = self.eval_format_impl(second_fmt, pairs)?;
            Val::String(format_with_vec(first_fmt.as_str(), res.cast_to_array())?.into())
        } else {
            format
        })
    }

    fn eval_format_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::format_exp);
        let mut pairs = token.into_inner();
        let format = self.eval_range_exp(pairs.next().unwrap())?;
        self.eval_format_impl(format, pairs)
    }

    fn eval_mult(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::multiplicative_exp);
        let mut pairs = token.into_inner();
        let mut res = self.eval_format_exp(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            let Some(fun) = ArithmeticPred::get(op.as_str()) else {
                panic!(
                    "can't find arithmetic function for operator: {}",
                    op.as_str()
                )
            };

            let postfix = pairs.next().unwrap();
            let right_op = self.eval_format_exp(postfix)?;
            res = fun(res, right_op)?;
        }

        Ok(res)
    }

    fn eval_additive(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::additive_exp);

        let mut pairs = token.into_inner();
        let mut res = self.eval_mult(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            //check_rule!(op, Rule::additive_op); plus or minus
            let Some(fun) = ArithmeticPred::get(op.as_str()) else {
                panic!()
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_mult(mult)?;
            res = fun(res, right_op)?;
        }

        Ok(res)
    }

    fn eval_split_special_case(
        &mut self,
        token: Pair<'a>,
        input: Val,
    ) -> ParserResult<Vec<String>> {
        let mut res_vec = vec![];
        let mut parts = String::new();
        let input_str = input.cast_to_string();
        let characters = input_str.chars();

        // filtered_elements.join("")
        for ch in characters {
            self.variables
                .set_ps_item(Val::String(ch.to_string().into()));

            let b = match self.eval_script_block(
                &ScriptBlock(token.as_str().to_string()),
                &Val::String(ch.to_string().into()),
            ) {
                Err(er) => {
                    self.errors.push(er);
                    false
                }
                Ok(b) => b,
            };

            if b {
                res_vec.push(parts);
                parts = String::new();
            } else {
                parts.push(ch);
            }
        }
        self.variables.reset_ps_item();
        Ok(res_vec)
    }
    fn eval_comparison_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::comparison_exp);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        // we need to handle strange case. -split and -join can be invoke without
        // previous expression, eg. "-join 'some'"
        let mut res = if token.as_rule() == Rule::additive_exp {
            self.eval_additive(token)?
        } else {
            Val::Null
        };

        while let Some(op) = pairs.next() {
            let Some(fun) = StringPred::get(op.as_str()) else {
                panic!("no operator: {}", op.as_str())
            };

            let token = pairs.next().unwrap();
            let right_op = match token.as_rule() {
                Rule::script_block_expression => {
                    let mut pairs = token.into_inner();
                    let token = pairs.next().unwrap();
                    check_rule!(token, Rule::script_block_inner);

                    let mut pairs = token.into_inner();
                    let mut token = pairs.next().unwrap();
                    if token.as_rule() == Rule::param_block {
                        //skip for now
                        token = pairs.next().unwrap();
                    }
                    check_rule!(token, Rule::script_block);
                    return Ok(Val::Array(
                        self.eval_split_special_case(token, res)?
                            .into_iter()
                            .map(|s| Val::String(s.into()))
                            .collect::<Vec<_>>(),
                    ));
                }
                Rule::additive_exp => self.eval_additive(token)?,
                _ => {
                    panic!("eval_comparison_exp not implemented: {:?}", token.as_rule());
                }
            };
            log::trace!("res: {:?}, right_op: {:?}", &res, &right_op);
            res = fun(res, right_op)?;
            log::trace!("res: {:?}", &res);
        }

        Ok(res)
    }

    fn eval_bitwise_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::bitwise_exp);

        let mut pairs = token.into_inner();
        let mut res = self.eval_comparison_exp(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            check_rule!(op, Rule::bitwise_operator);
            let Some(fun) = BitwisePred::get(op.as_str()) else {
                panic!()
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_comparison_exp(mult)?;
            res = fun(res, right_op);
        }

        Ok(res)
    }

    fn parse_long_command(&mut self, token: Pair<'a>) -> ParserResult<(String, Vec<CommandElem>)> {
        check_rule!(token, Rule::long_command);

        let mut pairs = token.into_inner();
        let command_name_token = pairs.next().unwrap();
        let command_name = command_name_token.as_str();

        let mut args = vec![];
        for command_element_token in pairs {
            let token_string = command_element_token.as_str().to_string();
            match command_element_token.as_rule() {
                Rule::command_argument => {
                    let arg_token = command_element_token.into_inner().next().unwrap();
                    let arg = match arg_token.as_rule() {
                        Rule::array_literal_exp => self.eval_array_literal_exp(arg_token)?,
                        Rule::parenthesized_expression => {
                            let token = arg_token.into_inner().next().unwrap();
                            self.eval_pipeline(token)?
                        }
                        _ => Val::String(arg_token.as_str().to_string().into()),
                    };
                    args.push(CommandElem::Argument(arg));
                }
                Rule::command_parameter => args.push(CommandElem::Parameter(token_string)),
                Rule::argument_list => args.push(CommandElem::ArgList(token_string)),
                Rule::redirection => { //todo: implement redirection
                }
                Rule::stop_parsing => { //todo: stop parsing
                }
                Rule::script_block_expression => args.push(CommandElem::Argument(
                    self.parse_script_block_expression(command_element_token)?,
                )),
                _ => panic!(
                    "eval_command not implemented: {:?}",
                    command_element_token.as_rule()
                ),
            }
        }
        Ok((command_name.to_string(), args))
    }

    fn parse_script_block_command(
        &mut self,
        token: Pair<'a>,
    ) -> ParserResult<(String, Vec<CommandElem>)> {
        check_rule!(token, Rule::script_block_command);

        let mut pairs = token.into_inner();
        let command_name_token = pairs.next().unwrap();
        let command_name = command_name_token.as_str();
        let command_element_token = pairs.next().unwrap();

        check_rule!(command_element_token, Rule::script_block_expression);
        let script_block = self.parse_script_block_expression(command_element_token)?;
        Ok((
            command_name.to_string(),
            vec![CommandElem::Argument(script_block)],
        ))
    }

    fn eval_command(&mut self, token: Pair<'a>, input: Option<Val>) -> ParserResult<Val> {
        check_rule!(token, Rule::command);

        let mut pairs = token.into_inner();
        //unfortunately I can't make long_command silent

        let mut args = if let Some(v) = input {
            vec![CommandElem::Argument(v)]
        } else {
            Vec::new()
        };

        let command = pairs.next().unwrap();
        let (command_name, mut command_args) = match command.as_rule() {
            Rule::script_block_command => self.parse_script_block_command(command)?,
            Rule::foreach_command => Err(ParserError::NotImplemented("foreach_command".into()))?,
            Rule::long_command => self.parse_long_command(command)?,
            Rule::invocation_command => {
                Err(ParserError::NotImplemented("invocation_command".into()))?
            }
            _ => panic!("eval_command not implemented: {:?}", command.as_rule()),
        };
        args.append(&mut command_args);

        let CommandOutput {
            val,
            stream_message,
        } = Command::execute(self, command_name.as_str(), args)?;

        if let Some(msg) = stream_message {
            self.stream.push(msg);
        }

        Ok(val.unwrap_or_default())
    }

    fn eval_redirected_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::redirected_expression);

        let expression_token = token.into_inner().next().unwrap();
        //todo: handle redirections

        self.eval_expression(expression_token)
    }

    fn eval_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::expression);
        let token_string = token.as_str().to_string();

        let mut pairs = token.into_inner();
        let mut res = self.eval_bitwise_exp(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            check_rule!(op, Rule::logical_operator);
            let Some(fun) = LogicalPred::get(op.as_str()) else {
                panic!()
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_bitwise_exp(mult)?;
            res = Val::Bool(fun(res, right_op));
        }
        self.tokens
            .push(Token::Expression(token_string, res.clone().into()));

        Ok(res)
    }

    fn safe_eval_pipeline_statement(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline_statement);
        let token = token.into_inner().next().unwrap();
        self.safe_eval_pipeline(token)
    }

    fn eval_pipeline_statement(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline_statement);
        let token = token.into_inner().next().unwrap();
        self.eval_pipeline(token)
    }

    fn eval_pipeline_tail(&mut self, token: Pair<'a>, input: Val) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline_tail);
        let mut arg = input;
        let mut pairs = token.into_inner();

        while let Some(token) = pairs.next() {
            arg = self.eval_command(token, Some(arg))?;
        }

        Ok(arg)
    }
    fn eval_pipeline(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        if let Rule::assignment_exp = token.as_rule() {
            return self.eval_assigment_exp(token);
        }

        let result: Val = match token.as_rule() {
            Rule::redirected_expression => self.eval_redirected_expression(token)?,
            Rule::command => self.eval_command(token, None)?,
            _ => {
                panic!("eval_pipeline not implemented: {:?}", token.as_rule());
            }
        };

        if let Some(token) = pairs.next() {
            match token.as_rule() {
                Rule::pipeline_tail => Ok(self.eval_pipeline_tail(token, result)?),
                _ => {
                    panic!("eval_pipeline not implemented: {:?}", token.as_rule());
                }
            }
        } else {
            Ok(result)
        }
    }

    fn safe_eval_pipeline(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        let res = self.eval_pipeline(token);

        let v = match res {
            Ok(val) => val,
            Err(err) => {
                self.errors.push(err);
                Val::Null
            }
        };

        Ok(v)
    }

    fn eval_cast_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::cast_expression);

        let mut pairs = token.into_inner();
        let type_token = pairs.next().unwrap();
        let val_type = self.eval_type_literal(type_token)?;

        let token = pairs.next().unwrap();
        let mut res = match token.as_rule() {
            Rule::parenthesized_expression => {
                let token = token.into_inner().next().unwrap();
                self.safe_eval_pipeline(token)?
            }
            Rule::range_exp => self.eval_range_exp(token)?,
            Rule::unary_exp => self.eval_unary_exp(token)?,
            _ => {
                panic!(
                    "eval_cast_expression not implemented: {:?}",
                    token.as_rule()
                );
            }
        };

        Ok(res.cast(val_type)?)
    }

    fn eval_assigment_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::assignment_exp);

        let mut pairs = token.into_inner();
        let variable_token = pairs.next().unwrap();
        let var_name = Self::parse_variable(variable_token)?;
        let var = self.variables.get(&var_name).unwrap_or_default();

        let assignement_op = pairs.next().unwrap();

        //get operand
        let op = assignement_op.into_inner().next().unwrap();
        let pred = ArithmeticPred::get(op.as_str());

        let right_token = pairs.next().unwrap();
        let right_result = self.eval_pipeline(right_token.clone());

        match right_result {
            Ok(right_operand) => {
                let Some(pred) = pred else { panic!() };
                let op_result = pred(var, right_operand)?;
                self.variables.set(&var_name, op_result.clone())?;
                Ok(Val::Null)
            }
            Err(err) => {
                self.errors.push(err);
                Ok(Val::String(
                    format!("{} = {}", var_name, right_token.as_str()).into(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;

    #[test]
    fn comment_and_semicolon() {
        let input = r#"
# This is a single line comment
$a = 1; $b = 2; Write-Output $a

Write-Output "Hello"  # Another comment

<#
    This is a
    multi-line block comment
#>
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn while_loop() {
        let input = r#"
while ($true) {
    if ($someCondition) {
        break
    }
    # other code
}
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn foreach_loop() {
        let input = r#"
foreach ($n in $numbers) {
    Write-Output $n
}
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn for_loop() {
        let input = r#"
# Comma separated assignment expressions enclosed in parentheses.
for (($i = 0), ($j = 0); $i -lt 10; $i++)
{
    "`$i:$i"
    "`$j:$j"
}
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn switch() {
        let input = r#"
switch ($var) {
    "a" { Write-Output "A" }
    1 { Write-Output "One" }
    default { Write-Output "Other" }
}
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn functions() {
        let input = r#"
function Get-Square {
    param($x)
    return $x * $x
}

function Say-Hello {
    Write-Output "Hello"
}
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn if_expression() {
        let input = r#"
$x="hello"
        Write-Host $x
        $y = 42
        Start-Process "notepad.exe"

        $x = 42
if ($x -eq 1) {
    Write-Output "One"
} elseif ($x -eq 2) {
    Write-Output "Two"
} else {
    Write-Output "Other"
}
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn command() {
        let input = r#"
Get-Process | Where-Object { $_.CPU -gt 100 }
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn range() {
        let input = r#"
$numbers = 1..5
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn literals() {
        let input = r#"
$hex = 0xFF

$name = "Alice"
$msg = "Hello, $name. Today is $day."
$escaped = "She said: `"Hi`""
$literal = 'Hello, $name'
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn floats() {
        let input = r#"
    $pi = 3.1415
$half = .5
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn arrays() {
        let input = r#"
$a = 1, 2, 3
$b = @("one", "two", "three")
$c = @(1, 2, @(3, 4))
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn static_method_call() {
        let input = r#"
[Threading.Thread]::Sleep(399)
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn neg_pipeline() {
        let input = r#"
-not $input | Where-Object { $_ -gt 5 }
"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn amsi_fail() {
        let input = r#"
#Matt Graebers second Reflection method 
$VMRviwsbtehQfPtxbt=$null;
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Âmsí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))"

"#;

        let _ = PowerShellSession::parse(Rule::program, input).unwrap();
    }
}
