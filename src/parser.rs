mod command;
mod predicates;
mod value;
mod variables;

use command::PsCommand;
use pest::Parser;
use pest_derive::Parser;
use predicates::{ArithmeticPred, ComparisonPred, ReplacePred, TypeCheckPred};
use thiserror_no_std::Error;
pub use value::{Val, ValType};
use variables::Variables;
use predicates::StringPred;

type PestError = pest::error::Error<Rule>;
type Pair<'i> = ::pest::iterators::Pair<'i, Rule>;
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
}

impl<'a> PowerShellParser {
    pub fn new() -> Self {
        Self {
            variables: Variables::new(),
        }
    }

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
                        res = self.eval_pipeline_statement(token)?;
                    }
                    Rule::EOI => {
                        break;
                    }
                    _ => {
                        println!("not implemented: {:?}", token.as_rule());
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
                        println!("not implemented: {:?}", token.as_rule());
                        panic!()
                    }
                }
            }
        }

        Ok(str_res)
    }

    pub fn eval_statements(&mut self, token: Pair<'a>) -> ParserResult<String> {
        let mut pairs = token.into_inner();
        let mut str_res = String::new();

        for token in pairs {
            //self.parse_statement(pair)?;
            match token.as_rule() {
                Rule::pipeline_statement => {
                    //println!("Assignment: {}", token.as_str());
                    str_res.push_str(&self.eval_pipeline_statement(token)?.cast_to_string());
                }
                _ => {
                    println!("not implemented: {:?}", token.as_rule());
                    panic!()
                }
            }
        }
        Ok(str_res)
    }

    // fn eval_cast_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
    //     check_rule!(token, Rule::cast_expression);
    //     let mut tokens = token.into_inner();

    //     let type_name_token = tokens.next().expect("Failed to get token");
    //     check_rule!(type_name_token, Rule::type_name);

    //     let val_type = ValType::cast(type_name_token.as_str())?;

    //     let expression = tokens.next().expect("Failed to get token");
    //     let mut val = match expression.as_rule() {
    //         Rule::expression => self.eval_expression(expression)?,
    //         Rule::number => self.eval_num(expression)?,
    //         Rule::postfix_expr => self.eval_postfix(expression)?,
    //         _ => {
    //             println!("token_rule: {:?}", expression.as_rule());
    //             todo!()
    //         }
    //     };
    //     Ok(val.cast(val_type)?)
    // }

    fn eval_args(&mut self, token: Pair<'a>) -> Option<Vec<Val>> {
        check_rule!(token, Rule::method_invocation);
        let token = token.into_inner().next().unwrap();
        check_rule!(token, Rule::argument_list);

        //println!("argument_list {:?}", token.as_rule());
        //println!("argument_list {:?}", token.as_str());

        let mut args = vec![];
        for token in token.into_inner() {
            match token.as_rule() {
                Rule::expression => {
                    if let Ok(arg) = self.eval_expression(token) {
                        args.push(arg);
                    }
                }
                _ => {
                    println!("eval_expression not implemented: {:?}", token.as_rule());
                }
            }
        }
        log::trace!("args {:?}", args);
        Some(args)
    }

    fn eval_command(&mut self, token: Pair<'a>) -> Option<Val> {
        check_rule!(token, Rule::dot_member);
        let token = token.into_inner().next().unwrap();
        self.eval_expression(token).ok()
    }

    fn eval_string_literal(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::string_literal);
        let mut pair = token.into_inner();
        let token= pair.next().unwrap();

        let res = match token.as_rule() {
            Rule::expandable_string_literal => {
                let mut res_str = String::new();
                let mut pairs = token.into_inner();
                while let Some(token) = pairs.next() {
                    let s = match token.as_rule() {
                        Rule::variable => self.get_variable(token)?.cast_to_string(),
                        Rule::sub_expression => self.eval_statements(token)?,
                        _ => token.as_str().to_string(),
                    };
                    res_str.push_str(s.as_str());
                }
                res_str
            },
            //Rule::expandable_multiline_string_literal => self.eval_expression(token)?,
            Rule::singlequated_string_literal => {
                if let Some(stripped_prefix) = token.as_str().to_string().strip_prefix("'") {
                    if let Some(stripped_suffix) = stripped_prefix.to_string().strip_suffix("'") {
                        stripped_suffix.to_string()
                    } else {
                        panic!("no suffix")
                    }
                } else {
                    panic!("no prefix")
                }
            },
            //Rule::singlequated_multiline_string_literal => self.eval_expression(token)?,
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
            }
        };
        Ok(Val::String(res))
    }

     fn get_variable(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::variable);
        let (var_name, scope) = self.parse_variable(token)?;
        Ok(self.variables.get(&var_name, scope))
     }

    fn parse_variable(&mut self, token: Pair<'a>) -> ParserResult<(String, Option<String>)> {
        check_rule!(token, Rule::variable);
        let mut pair = token.into_inner();
        let mut token= pair.next().unwrap();

        Ok(if token.as_rule() == Rule::special_variable {
            (token.as_str().to_ascii_lowercase(), None)
        } else {
            //check if scope is present
            let scope = if token.as_rule() == Rule::scope_keyword {
                let scope = token.as_str().to_ascii_lowercase();
                token = pair.next().unwrap();
                Some(scope)
            } else {
                None
            };
            check_rule!(token, Rule::var_name);
            (token.as_str().to_ascii_lowercase(), scope)
        })
    }

    fn eval_expression_with_unary_operator(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::expression_with_unary_operator  );
        let mut pair = token.into_inner();
        let token= pair.next().unwrap();

        let res = match token.as_rule() {
            Rule::pre_inc_expression => {
                let token = token.into_inner().next().unwrap();
                let mut primary = self.eval_primary_expression(token)?;
                primary.pre_inc()?;
                primary
            }
            Rule::pre_dec_expression => {
                let token = token.into_inner().next().unwrap();
                let primary = self.eval_primary_expression(token)?;
                todo!();//primary.pre_dec()?;
                primary
            }
            Rule::cast_expression => self.eval_cast_expression(token)?,
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

        let member_access= pairs.next().unwrap();
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
        check_rule!(token, Rule::access );
        let mut pairs = token.into_inner();
        let token= pairs.next().unwrap();

        let mut value = self.eval_value(token)?; 

        while let Some(token) = pairs.next() {
            match token.as_rule() {
                Rule::method_invocation =>  {
                    let (method_name, args) = self.eval_method_invokation(token)?;
                    if let Some(result) = PsCommand::call(value.clone(), method_name.as_str(), args) {
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
        check_rule!(token, Rule::primary_expression );
        let mut pair = token.into_inner();
        let token= pair.next().unwrap();

        let res = match token.as_rule() {
            //Rule::post_inc_expression => self.eval_expression(token)?,
            //Rule::post_dec_expression => self.eval_expression(token)?,
            Rule::access => self.eval_access(token)?,
            Rule::value => self.eval_value(token)?,
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                println!("token.rule(): {:?}", token.as_str());
                panic!()
            }
        };

        Ok(res)
    }

    fn eval_type_literal(&mut self, token: Pair<'a>) -> ParserResult<ValType> {
        check_rule!(token, Rule::type_literal );

        let token = token.into_inner().next().unwrap();
        check_rule!(token, Rule::type_spec );
        let res = ValType::cast(token.as_str())?;
        Ok(res)
    }

    fn eval_value(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::value);
        let mut pair = token.into_inner();
        let token= pair.next().unwrap();

        let res = match token.as_rule() {
            Rule::parenthesized_expression => {
                let token = token.into_inner().next().unwrap();
                self.eval_pipeline(token)?
            }
            //Rule::array_expression => self.eval_expression(token)?,
            //Rule::script_block_expression => self.eval_expression(token)?,
            //Rule::hash_literal_expression => self.eval_expression(token)?,
            Rule::string_literal => self.eval_string_literal(token)?,
            Rule::number_literal => self.eval_number_literal(token)?,
            Rule::type_literal => Val::init(self.eval_type_literal(token)?)?,
            Rule::variable => self.get_variable(token)?,
            Rule::sub_expression => Val::String(self.eval_statements(token)?),
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
            }
        };

        Ok(res)
    }

    fn eval_number_literal(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::number_literal);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();
        let mut val = self.eval_number(token)?;
        if let Some(unit) = pairs.next() {
            let unit = unit.as_str().to_ascii_lowercase();
            let unit_int = match unit.as_str(){
                "k" => 1024,
                "m" => 1024*1024,
                "g" => 1024*1024*1024,
                "t" => 1024*1024*1024*1024,
                "p" => 1024*1024*1024*1024*1024,
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
                println!("int_val: \'{}\'", int_val.as_str());
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

    fn eval_command_call(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::command_call);
        let mut pairs = token.into_inner();
        let ident = pairs.next().unwrap();

        let res = match ident.as_rule() {
            Rule::command_ident => Val::String(ident.as_str().to_string()),
            _ => {
                println!("eval_command_call token.rule(): {:?}", ident.as_rule());
                panic!()
            }
        };

        while let Some(_exp) = pairs.next() {
            todo!()
        }
        Ok(res)
    }

    fn eval_unary_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::unary_exp);
        let mut token = token.into_inner().next().unwrap();
        match token.as_rule() {
            Rule::expression_with_unary_operator => self.eval_expression_with_unary_operator(token),
            Rule::primary_expression => self.eval_primary_expression(token),
            _ => {
                println!("eval_command_call token.rule(): {:?}", token.as_rule());
                panic!()
            }
        }
    }

    fn eval_array_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::array_exp);
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
        check_rule!(token, Rule::range_exp);
        let mut pairs = token.into_inner();
        let mut res = self.eval_array_exp(pairs.next().unwrap())?;
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

    fn eval_format_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::format_exp);
        let mut pairs = token.into_inner();
        let mut res = self.eval_range_exp(pairs.next().unwrap())?;
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

    fn eval_mult(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::multiplicative_exp);
        let mut pairs = token.into_inner();
        let mut res = self.eval_format_exp(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
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
            let Some(fun) = ArithmeticPred::get(op.as_str()) else {
                panic!()
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_mult(mult)?;
            res = fun(res, right_op);
        }

        Ok(res)
    }

    fn eval_comparison_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::comparison_exp);

        let mut pairs = token.into_inner();
        let mut res = self.eval_additive(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            let Some(fun) = StringPred::get(op.as_str()) else {
                panic!()
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_additive(mult)?;
            res = fun(res, right_op)?;
        }

        Ok(res)
    }

    fn eval_bitwise_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::bitwise_exp);

        let mut pairs = token.into_inner();
        let mut res = self.eval_comparison_exp(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            let Some(fun) = ArithmeticPred::get(op.as_str()) else {
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
            let Some(fun) = ArithmeticPred::get(op.as_str()) else {
                panic!()
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_bitwise_exp(mult)?;
            res = fun(res, right_op);
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
            Rule::expression => self.eval_expression(token)?,
            _ => {
                println!("eval_pipeline not implemented: {:?}", token.as_rule());
                panic!();
            }
        };

        Ok(res)
    }

    // fn eval_replace_operator_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
    //     check_rule!(token, Rule::replace_operator_exp);

    //     let mut pairs = token.into_inner();
    //     let first_operand = pairs.next().unwrap();
    //     check_rule!(first_operand, Rule::additive_exp);
    //     let base = self.eval_additive(first_operand)?;

    //     let operator_token = pairs.next().unwrap();
    //     check_rule!(operator_token, Rule::replace_op);

    //     let second_operand = pairs.next().unwrap();
    //     check_rule!(second_operand, Rule::additive_exp);
    //     let from = self.eval_additive(second_operand)?;

    //     let to = if let Some(third_operand) = pairs.next() {
    //         check_rule!(third_operand, Rule::additive_exp);
    //         self.eval_additive(third_operand)?
    //     } else {
    //         Val::Null
    //     };

    //     let Some(replace_fn) = ReplacePred::get(operator_token.as_str()) else {
    //         panic!();
    //     };
    //     Ok(Val::String(replace_fn(base, from, to)))
    // }

    // fn eval_cmp_operator_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
    //     check_rule!(token, Rule::cmp_operator_exp);

    //     let mut pairs = token.into_inner();
    //     let first_operand = pairs.next().unwrap();
    //     check_rule!(first_operand, Rule::additive_exp);
    //     let v1 = self.eval_additive(first_operand)?;

    //     let operator_token = pairs.next().unwrap();
    //     check_rule!(operator_token, Rule::cmp_op);

    //     let second_operand = pairs.next().unwrap();
    //     check_rule!(second_operand, Rule::additive_exp);
    //     let v2 = self.eval_additive(second_operand)?;

    //     //let token = operator_token.into_inner().next().unwrap();

    //     let Some(cmp_fn) = ComparisonPred::get(operator_token.as_str()) else {
    //         panic!();
    //     };
    //     Ok(Val::Bool(cmp_fn(v1, v2)))
    // }

    // fn eval_typecheck_operator_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
    //     check_rule!(token, Rule::typecheck_operator_exp);

    //     let mut pairs = token.into_inner();
    //     let first_operand = pairs.next().unwrap();
    //     check_rule!(first_operand, Rule::additive_exp);
    //     let var = self.eval_additive(first_operand)?;

    //     let operator_token = pairs.next().unwrap();
    //     check_rule!(operator_token, Rule::type_check_op);

    //     let second_operand = pairs.next().unwrap();
    //     check_rule!(second_operand, Rule::type_name);
    //     let ttype = ValType::cast(second_operand.as_str())?;

    //     let Some(typecast_fn) = TypeCheckPred::get(operator_token.as_str()) else {
    //         panic!();
    //     };
    //     Ok(Val::Bool(typecast_fn(var, ttype)))
    // }

    fn eval_cast_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::cast_expression);
        let mut res = Val::default();

        let mut pairs = token.into_inner();
        let type_token = pairs.next().unwrap();
        let val_type = self.eval_type_literal(type_token)?;

        let unary_token = pairs.next().unwrap();
        let mut val = self.eval_unary_exp(unary_token)?;

        Ok(val.cast(val_type)?)
    }

    fn expand_string(&mut self, token: Pair<'a>) -> ParserResult<String> {
        //check_rule!(token, Rule::expandable_string_content);
        Ok(match token.as_rule() {
            Rule::variable => self.get_variable(token)?.cast_to_string(),
            Rule::sub_expression => self.eval_statements(token)?,
            _ => String::from(token.as_str())
        })
    }

    // fn expand_strings(input: &str) -> ParserResult<String> {
    //     let mut pairs = PowerShellParser::parse(Rule::program, input)?;
    //     let program_token = pairs.next().expect("");

    //     if let Rule::program = program_token.as_rule() {
    //         let mut pairs = program_token.into_inner();
    //         let token = pairs.next().unwrap();

    //         if let Rule::expression  = token.as_rule() {
    //             let mut pairs = token.into_inner();
    //             let token = pairs.next().unwrap();

    //             if let Rule::assignment_exp  = token.as_rule() {
    //                 let mut pairs = token.into_inner();
    //                 let token = pairs.next().unwrap();
    //                 let token = pairs.next().unwrap();

    //                 if let Rule::expression  = token.as_rule() {
    //                     let mut pairs = token.into_inner();
    //                     let token = pairs.next().unwrap();

    //                     if let Rule::additive_exp = token.as_rule() {
    //                         let mut pairs = token.into_inner();
    //                         let token = pairs.next().unwrap();
    //                         if let Rule::multiplicative_exp  = token.as_rule() {
    //                             let mut pairs = token.into_inner();
    //                             let token = pairs.next().unwrap();

    //                             if let Rule::postfix_expr  = token.as_rule() {
    //                                 let mut pairs = token.into_inner();
    //                                 let token = pairs.next().unwrap();

    //                                 if let Rule::expandable_string_content  =
    // token.as_rule() {
    // //println!("Expandable: {}", token.as_str());
    // return Ok(self.expand_string(token)?.to_string());
    // }                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     todo!()
    // }

    fn eval_assigment_exp(&mut self, token: Pair<'a>) -> ParserResult<()> {
        check_rule!(token, Rule::assignment_exp);

        let mut pairs = token.into_inner();
        let variable_token = pairs.next().unwrap();
        let (var_name, scope) = self.parse_variable(variable_token)?;
        let var = self.variables.get(&var_name, scope);
        
        let assignement_op = pairs.next().unwrap();

        //get operand
        let op = assignement_op.into_inner().next().unwrap();
        let pred = ArithmeticPred::get(op.as_str());

        let expression_token = pairs.next().unwrap();
        let expression_result = self.eval_expression(expression_token)?;

        let Some(pred) = pred else { panic!() };

        self.variables
            .set(&var_name, None, pred(var, expression_result));

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

    //#[test]
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
$bin = 0b1101

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
    fn amsi_fail() {
        let input = r#"
#Matt Graebers second Reflection method 
$VMRviwsbtehQfPtxbt=$null;
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Âmsí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))"

"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }
}
