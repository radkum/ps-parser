mod command;
mod predicates;
mod value;
mod variables;

use command::PsCommand;
use pest::Parser;
use pest_derive::Parser;
use predicates::{ArithmeticPred, BitwisePred, LogicalPred, StringPred};
use thiserror_no_std::Error;
pub use value::{Val, ValType};
use variables::Variables;

type PestError = pest::error::Error<Rule>;
type Pair<'i> = ::pest::iterators::Pair<'i, Rule>;
type Pairs<'i> = ::pest::iterators::Pairs<'i, Rule>;
use predicates::OpError;
use value::ValError;

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("PestError: {0}")]
    PestError(PestError),

    #[error("ValError: {0}")]
    ValError(ValError),

    #[error("OperatorError: {0}")]
    OpError(OpError),
}

impl From<PestError> for ParserError {
    fn from(value: PestError) -> Self {
        Self::PestError(value)
    }
}

impl From<ValError> for ParserError {
    fn from(value: ValError) -> Self {
        Self::ValError(value)
    }
}

impl From<OpError> for ParserError {
    fn from(value: OpError) -> Self {
        Self::OpError(value)
    }
}

type ParserResult<T> = core::result::Result<T, ParserError>;

macro_rules! check_rule {
    ($pair:expr, $rule:pat) => {
        if !matches!($pair.as_rule(), $rule) {
            println!("rule: {:?}", $pair.as_rule());
            panic!()
        }
    };
}

#[derive(Parser)]
#[grammar = "powershell.pest"]
pub(crate) struct PowerShellParser {
    variables: Variables,
    errors: Vec<ParserError>,
}

impl<'a> PowerShellParser {
    pub fn new() -> Self {
        Self {
            variables: Variables::new(),
            errors: vec![],
        }
    }

    #[allow(unused_mut)]
    pub fn errors(self) -> Vec<ParserError> {
        self.errors
    }

    #[allow(unused_mut)]
    pub fn safe_eval(&mut self, input: &str) -> ParserResult<String> {
        let mut pairs = PowerShellParser::parse(Rule::program, input)?;
        let program_token = pairs.next().expect("");
        let mut res = Val::default();

        if let Rule::program = program_token.as_rule() {
            let pairs = program_token.into_inner();
            for token in pairs {
                //self.parse_statement(pair)?;
                match token.as_rule() {
                    Rule::pipeline_statement => {
                        //println!("Assignment: {}", token.as_str());
                        res = self.eval_pipeline_statement(token).unwrap_or_default();
                    }
                    Rule::EOI => {
                        break;
                    }
                    Rule::pipeline => {
                        //println!("Assignment: {}", token.as_str());
                        res = self.eval_pipeline(token).unwrap_or_default();
                    }
                    _ => {
                        println!("safe_eval not implemented: {:?}", token.as_rule());
                    }
                }
            }
        }

        Ok(res.cast_to_string())
    }

    pub fn evaluate(&mut self, input: &str) -> ParserResult<String> {
        let mut pairs = PowerShellParser::parse(Rule::program, input)?;
        let program_token = pairs.next().expect("");
        let mut str_res = String::new();

        if let Rule::program = program_token.as_rule() {
            let pairs = program_token.into_inner();
            for token in pairs {
                //self.parse_statement(pair)?;
                match token.as_rule() {
                    Rule::pipeline_statement => {
                        //println!("Assignment: {}", token.as_str());
                        str_res.push_str(&self.eval_pipeline_statement(token)?.cast_to_string());
                        str_res.push_str("\n");
                    }
                    Rule::EOI => {
                        break;
                    }
                    _ => {
                        println!(" evaluate not implemented: {:?}", token.as_rule());
                        panic!()
                    }
                }
            }
        }

        Ok(str_res)
    }

    pub fn eval_statements(&mut self, token: Pair<'a>) -> ParserResult<Vec<Val>> {
        let pairs = token.into_inner();
        let mut v = vec![];

        for token in pairs {
            //self.parse_statement(pair)?;
            match token.as_rule() {
                Rule::pipeline_statement => {
                    //println!("Assignment: {}", token.as_str());
                    v.push(self.eval_pipeline_statement(token)?);
                }
                Rule::pipeline => {
                    //println!("Assignment: {}", token.as_str());
                    v.push(self.eval_pipeline(token)?);
                }
                _ => {
                    println!("eval statemets not implemented: {:?}", token.as_rule());
                    panic!()
                }
            }
        }
        //Ok(Val::Array(Box::new(v)))
        Ok(v)
    }

    fn parse_dq(&mut self, token: Pair<'a>) -> ParserResult<String> {
        let mut res_str = String::new();
        let mut pairs = token.into_inner();
        while let Some(token) = pairs.next() {
            let token = token.into_inner().next().unwrap();
            let s = match token.as_rule() {
                Rule::variable => self.get_variable(token)?.cast_to_string(),
                Rule::sub_expression => {
                    Val::Array(Box::new(self.eval_statements(token)?)).cast_to_string()
                }
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

        let res = match token.as_rule() {
            Rule::doublequoted_string_literal => self.parse_dq(token)?,
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
            Rule::doublequoted_multiline_string_literal => self.parse_dq(token)?,
            Rule::singlequoted_multiline_string_literal => {
                let mut res_str = String::new();
                let mut pairs = token.into_inner();
                while let Some(token) = pairs.next() {
                    res_str.push_str(token.as_str());
                }
                res_str
            }
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
            }
        };
        Ok(Val::String(res))
    }

    fn get_variable(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::variable);
        let var_name = self.parse_variable(token)?;
        Ok(self.variables.get(&var_name))
    }

    fn parse_variable(&mut self, token: Pair<'a>) -> ParserResult<String> {
        check_rule!(token, Rule::variable);
        let mut pair = token.into_inner();
        let token = pair.next().unwrap();

        Ok(match token.as_rule() {
            Rule::special_variable => token.as_str().to_string(),
            Rule::parenthesized_variable => {
                self.parse_variable(token.into_inner().next().unwrap())?
            }
            Rule::braced_variable => {
                let token = token.into_inner().next().unwrap();
                token.as_str().to_string()
            }
            Rule::scoped_variable => {
                let mut pairs = token.into_inner();
                let token = pairs.next().unwrap();

                if token.as_rule() == Rule::scope_keyword {
                    let scope = token.as_str().to_ascii_lowercase();
                    let token = pair.next().unwrap();
                    check_rule!(token, Rule::var_name);
                    format!("{}:{}", scope, token.as_str().to_string())
                } else {
                    token.as_str().to_string()
                }
            }
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
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
                let var_name = self.parse_variable(variable_token)?;
                let mut var = self.variables.get(&var_name);
                var.inc()?;

                self.variables.set(&var_name, var.clone());
                var
            }
            Rule::pre_dec_expression => {
                let variable_token = token.into_inner().next().unwrap();
                let var_name = self.parse_variable(variable_token)?;
                let mut var = self.variables.get(&var_name);
                var.dec()?;

                self.variables.set(&var_name, var.clone());
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
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
            }
        };

        Ok(res)
    }

    fn eval_argument_list(&mut self, token: Pair<'a>) -> ParserResult<Vec<Val>> {
        check_rule!(token, Rule::argument_list);
        let mut pairs = token.into_inner();

        let mut args = Vec::new();
        while let Some(token) = pairs.next() {
            args.push(self.eval_expression(token)?);
        }

        Ok(args)
    }

    fn eval_member_access(&mut self, token: Pair<'a>) -> ParserResult<String> {
        check_rule!(token, Rule::member_access);
        let member_name_token = token.into_inner().next().unwrap();
        let member_name = member_name_token.as_str().to_ascii_lowercase();

        Ok(member_name)
    }

    fn eval_method_invokation(&mut self, token: Pair<'a>) -> ParserResult<(String, Vec<Val>)> {
        check_rule!(token, Rule::method_invocation);
        let mut pairs = token.into_inner();

        let member_access = pairs.next().unwrap();
        check_rule!(member_access, Rule::member_access);
        let method_name = self.eval_member_access(member_access)?;

        let args = if let Some(token) = pairs.next() {
            check_rule!(token, Rule::argument_list);
            self.eval_argument_list(token)?
        } else {
            Vec::new()
        };

        Ok((method_name, args))
    }

    fn eval_access(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::access);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        let mut value = self.eval_value(token)?;

        while let Some(token) = pairs.next() {
            match token.as_rule() {
                Rule::method_invocation => {
                    let (method_name, args) = self.eval_method_invokation(token)?;
                    if let Some(result) = PsCommand::call(value.clone(), method_name.as_str(), args)
                    {
                        value = result;
                    }
                }
                //Rule::member_access => res.invoke(self.eval_method_invokation(token))?,
                //Rule::element_access => res.invoke(self.eval_method_invokation(token))?,
                _ => {
                    println!("token.rule(): {:?}", token.as_rule());
                    panic!()
                }
            }
        }

        Ok(value)
    }

    fn eval_primary_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::primary_expression);
        let mut pair = token.into_inner();
        let token = pair.next().unwrap();

        let res = match token.as_rule() {
            Rule::access => self.eval_access(token)?,
            Rule::value => self.eval_value(token)?,
            Rule::post_inc_expression => {
                let variable_token = token.into_inner().next().unwrap();
                let var_name = self.parse_variable(variable_token)?;
                let mut var = self.variables.get(&var_name);
                let var_to_return = var.clone();

                var.inc()?;
                self.variables.set(&var_name, var.clone());

                //if var_to_return.ttype() ==
                var_to_return
            }
            Rule::post_dec_expression => {
                let variable_token = token.into_inner().next().unwrap();
                let var_name = self.parse_variable(variable_token)?;
                let mut var = self.variables.get(&var_name);
                let var_to_return = var.clone();

                var.dec()?;
                self.variables.set(&var_name, var.clone());

                var_to_return
            }
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                println!("token.rule(): {:?}", token.as_str());
                panic!()
            }
        };

        Ok(res)
    }

    fn eval_type_literal(&mut self, token: Pair<'a>) -> ParserResult<ValType> {
        check_rule!(token, Rule::type_literal);

        let token = token.into_inner().next().unwrap();
        check_rule!(token, Rule::type_spec);
        let res = ValType::cast(token.as_str())?;
        Ok(res)
    }

    fn eval_script_block_body(&mut self, token: Pair<'a>) -> ParserResult<Vec<Val>> {
        check_rule!(token, Rule::script_block_body);

        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        Ok(match token.as_rule() {
            Rule::statements_block => self.eval_statements(token)?,
            _ => {
                println!("eval_script_block token.rule(): {:?}", token.as_rule());
                panic!()
            }
        })
    }

    fn eval_script_block(&mut self, token: Pair<'a>) -> ParserResult<Vec<Val>> {
        check_rule!(token, Rule::script_block);

        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        Ok(match token.as_rule() {
            Rule::script_block_body => self.eval_script_block_body(token)?,
            _ => {
                println!("eval_script_block token.rule(): {:?}", token.as_rule());
                panic!()
            }
        })
    }

    fn eval_script_block_expression(&mut self, token: Pair<'a>) -> ParserResult<Vec<Val>> {
        check_rule!(token, Rule::script_block_expression);

        let mut pairs = token.into_inner();
        let mut token = pairs.next().unwrap();
        if token.as_rule() == Rule::param_block {
            //todo parse param block
            token = pairs.next().unwrap();
        }

        check_rule!(token, Rule::script_block);
        self.eval_script_block(token)
    }

    fn eval_value(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::value);
        let mut pair = token.into_inner();
        let token = pair.next().unwrap();

        let res = match token.as_rule() {
            Rule::parenthesized_expression => {
                let token = token.into_inner().next().unwrap();
                self.eval_pipeline(token)?
            }
            Rule::string_literal => self.eval_string_literal(token)?,
            Rule::number_literal => self.eval_number_literal(token)?,
            Rule::type_literal => Val::init(self.eval_type_literal(token)?)?,
            Rule::variable => self.get_variable(token)?,
            Rule::sub_expression | Rule::array_expression => {
                let v = self.eval_statements(token)?;
                if v.len() == 1 && v[0].ttype() == ValType::Array {
                    v[0].clone()
                } else {
                    Val::Array(Box::new(v))
                }
            }
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
            }
        };

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
                println!("int_val: {}", int_val.as_str());
                Val::Int(int_val.as_str().parse::<i64>().unwrap())
            }
            Rule::hex_integer => {
                let int_val = token.into_inner().next().unwrap();
                Val::Int(i64::from_str_radix(int_val.as_str(), 16).unwrap())
            }
            //todo: parse float in proper way
            Rule::float => {
                println!("float: \'{}\'", token.as_str());
                Val::Float(token.as_str().trim().parse::<f64>().unwrap())
            }
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
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
                println!("eval_unary_exp token.rule(): {:?}", token.as_rule());
                panic!()
            }
        }
    }

    fn eval_array_literal_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::array_literal_exp);
        let mut arr = Vec::new();
        let mut pairs = token.into_inner();
        arr.push(self.eval_unary_exp(pairs.next().unwrap())?);
        while let Some(token) = pairs.next() {
            arr.push(self.eval_unary_exp(token)?);
        }

        Ok(if arr.len() == 1 {
            arr[0].clone()
        } else {
            Val::Array(Box::new(arr))
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
            .map(|i| Val::Int(i))
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
                Val::Array(Box::new(range(left, right)))
            }
            Rule::array_literal_exp => {
                let res = self.eval_array_literal_exp(token)?;
                if let Some(token) = pairs.next() {
                    let left = res.cast_to_int()?;
                    let right = self.eval_array_literal_exp(token)?.cast_to_int()?;
                    Val::Array(Box::new(range(left, right)))
                } else {
                    res
                }
            }
            _ => {
                println!("eval_range_exp not implemented: {:?}", token.as_rule());
                panic!();
            }
        };
        // while let Some(op) = pairs.next() {
        //     let Some(fun) = ArithmeticPred::get(op.as_str()) else {
        //         panic!()
        //     };

        //     let postfix = pairs.next().unwrap();
        //     let right_op = self.eval_format_exp(postfix)?;
        //     println!("{} {:?} {:?}", op.as_str(), res, right_op);
        //     res = fun(res, right_op);
        // }

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
                        if fmt_vec[i] != ('0' as u8) {
                            i += 1
                        } else {
                            fmt_vec[i] = digit + ('0' as u8);
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
                                            format!("{}", val.cast_to_string())
                                        }
                                    }
                                    Some(s) => strange_special_case(s, val.cast_to_int()?),
                                    None => format!("{}", val.cast_to_string()),
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
                                    _ => format!("{}", val.cast_to_string()),
                                },
                                None => format!("{{{}}}", token), /* leave as-is if index out of
                                                                   * bounds */
                            }
                        } else {
                            let index: usize =
                                Val::String(token.to_string()).cast_to_int()? as usize;
                            match args.get(index) {
                                Some(val) => format!("{}", val.cast_to_string()),
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
            Val::String(format_with_vec(first_fmt.as_str(), res.cast_to_array())?)
        } else {
            format
        })
    }

    fn eval_format_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::format_exp);
        let mut pairs = token.into_inner();
        let format = self.eval_range_exp(pairs.next().unwrap())?;
        Ok(self.eval_format_impl(format, pairs)?)
    }

    fn eval_mult(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::multiplicative_exp);
        let mut pairs = token.into_inner();
        let mut res = self.eval_format_exp(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            //check_rule!(op, Rule::multiplicative_op);
            let Some(fun) = ArithmeticPred::get(op.as_str()) else {
                panic!()
            };

            let postfix = pairs.next().unwrap();
            let right_op = self.eval_format_exp(postfix)?;
            println!("{} {:?} {:?}", op.as_str(), res, right_op);
            res = fun(res, right_op);
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
            res = fun(res, right_op);
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
        let mut characters = input_str.chars();
        while let Some(ch) = characters.next() {
            self.variables.set("$_", Val::String(ch.to_string()));
            let Ok(v) = self.eval_script_block_expression(token.clone()) else {
                return Ok(vec![]);
            };

            if v.len() != 1 || v[0].ttype() != ValType::Bool {
                return Ok(vec![]);
            }

            let Val::Bool(b) = v[0] else {
                return Ok(vec![]);
            };

            if b {
                res_vec.push(parts);
                parts = String::new();
            } else {
                parts.push(ch);
            }
        }

        self.variables.set("$_", Val::Null);
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
                    return Ok(Val::Array(Box::new(
                        self.eval_split_special_case(token, res)?
                            .into_iter()
                            .map(|s| Val::String(s))
                            .collect::<Vec<_>>(),
                    )));
                }
                Rule::additive_exp => self.eval_additive(token)?,
                _ => {
                    println!("eval_comparison_exp not implemented: {:?}", token.as_rule());
                    panic!();
                }
            };
            log::trace!("res: {:?}, right_op: {:?}", &res, &right_op);
            res = fun(res, right_op)?;
            println!("res: {:?}", &res);
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

    fn eval_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::expression);

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

        Ok(res)
    }

    fn eval_pipeline_statement(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline_statement);
        let token = token.into_inner().next().unwrap();
        self.eval_pipeline(token)
    }

    fn eval_pipeline(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        let res = match token.as_rule() {
            Rule::assignment_exp => {
                self.eval_assigment_exp(token)?;
                Val::default()
            }
            Rule::expression => match self.eval_expression(token) {
                Ok(val) => val,
                Err(err) => {
                    self.errors.push(err);
                    Val::Null
                }
            },
            _ => {
                println!("eval_pipeline not implemented: {:?}", token.as_rule());
                panic!();
            }
        };

        Ok(res)
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
                self.eval_pipeline(token)?
            }
            Rule::range_exp => self.eval_range_exp(token)?,
            Rule::unary_exp => self.eval_unary_exp(token)?,
            _ => {
                println!(
                    "eval_cast_expression not implemented: {:?}",
                    token.as_rule()
                );
                panic!();
            }
        };

        Ok(res.cast(val_type)?)
    }

    fn eval_assigment_exp(&mut self, token: Pair<'a>) -> ParserResult<()> {
        check_rule!(token, Rule::assignment_exp);

        let mut pairs = token.into_inner();
        let variable_token = pairs.next().unwrap();
        let var_name = self.parse_variable(variable_token)?;
        let var = self.variables.get(&var_name);

        let assignement_op = pairs.next().unwrap();

        //get operand
        let op = assignement_op.into_inner().next().unwrap();
        let pred = ArithmeticPred::get(op.as_str());

        let expression_token = pairs.next().unwrap();
        let expression_result = self.eval_expression(expression_token)?;

        let Some(pred) = pred else { panic!() };

        self.variables.set(&var_name, pred(var, expression_result));

        Ok(())
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

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
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

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn foreach_loop() {
        let input = r#"
foreach ($n in $numbers) {
    Write-Output $n
}
"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
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

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
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

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
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

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
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

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn command() {
        let input = r#"
Get-Process | Where-Object { $_.CPU -gt 100 }
"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn range() {
        let input = r#"
$numbers = 1..5
"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
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

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn floats() {
        let input = r#"
    $pi = 3.1415
$half = .5
"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn arrays() {
        let input = r#"
$a = 1, 2, 3
$b = @("one", "two", "three")
$c = @(1, 2, @(3, 4))
"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn static_method_call() {
        let input = r#"
[Threading.Thread]::Sleep(399)
"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn neg_pipeline() {
        let input = r#"
-not $input | Where-Object { $_ -gt 5 }
"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }

    #[test]
    fn amsi_fail() {
        let input = r#"
#Matt Graebers second Reflection method 
$VMRviwsbtehQfPtxbt=$null;
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Âmsí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))"

"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }
}
