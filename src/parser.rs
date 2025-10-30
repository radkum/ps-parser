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
use value::{Param, RuntimeObject, ScriptBlock, ValResult};
use variables::{Scope, SessionScope};
type ParserResult<T> = core::result::Result<T, ParserError>;
use error::ParserError;
type PestError = pest::error::Error<Rule>;
use pest::Parser;
use pest_derive::Parser;
use predicates::{ArithmeticPred, BitwisePred, LogicalPred, StringPred};
pub use script_result::{PsValue, ScriptResult};
pub use token::{Token, Tokens};
pub(crate) use value::{Val, ValType};
pub use variables::Variables;
use variables::{VarName, VariableError};

use crate::parser::command::CommandOutput;

type Pair<'i> = ::pest::iterators::Pair<'i, Rule>;
type Pairs<'i> = ::pest::iterators::Pairs<'i, Rule>;

pub(crate) const NEWLINE: &str = "\n";

macro_rules! unexpected_token {
    ($pair:expr) => {
        panic!("Unexpected token: {:?}", $pair.as_rule())
    };
}

macro_rules! check_rule {
    ($pair:expr, $rule:pat) => {
        if !matches!($pair.as_rule(), $rule) {
            panic!(
                "Unexpected token: {:?}, instead of {}",
                $pair.as_rule(),
                stringify!($rule)
            );
        }
    };
}

macro_rules! not_implemented {
    ($token:expr) => {
        Err(ParserError::NotImplemented(format!(
            "Not implemented: {:?}",
            $token.as_rule()
        )))
    };
}

#[derive(Default)]
pub(crate) struct Results {
    output: Vec<StreamMessage>,
    deobfuscated: Vec<String>,
}

impl Results {
    fn new() -> Self {
        Self {
            output: Vec::new(),
            deobfuscated: Vec::new(),
        }
    }
}

#[derive(Parser)]
#[grammar = "powershell.pest"]
pub struct PowerShellSession {
    variables: Variables,
    tokens: Tokens,
    errors: Vec<ParserError>,
    results: Vec<Results>,
}

impl Default for PowerShellSession {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> PowerShellSession {
    /// Creates a new PowerShell parsing session with default settings.
    ///
    /// The session is initialized with built-in variables like `$true`,
    /// `$false`, `$null`, and special variables like `$?` for error status
    /// tracking.
    ///
    /// # Returns
    ///
    /// A new `PowerShellSession` instance ready for script evaluation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ps_parser::PowerShellSession;
    ///
    /// let mut session = PowerShellSession::new();
    /// let result = session.safe_eval("$true").unwrap();
    /// assert_eq!(result, "True");
    /// ```
    pub fn new() -> Self {
        Self {
            variables: Variables::new(),
            tokens: Tokens::new(),
            errors: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Creates a new PowerShell session with the provided variables.
    ///
    /// This constructor allows you to initialize the session with a custom set
    /// of variables, such as environment variables or variables loaded from
    /// configuration files.
    ///
    /// # Arguments
    ///
    /// * `variables` - A `Variables` instance containing the initial variable
    ///   set.
    ///
    /// # Returns
    ///
    /// A new `PowerShellSession` instance with the provided variables.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ps_parser::{PowerShellSession, Variables};
    ///
    /// let env_vars = Variables::env();
    /// let mut session = PowerShellSession::new().with_variables(env_vars);
    /// let username = session.safe_eval("$env:USERNAME").unwrap();
    /// ```
    pub fn with_variables(mut self, variables: Variables) -> Self {
        self.variables = variables;
        self
    }

    /// Safely evaluates a PowerShell script and returns the output as a string.
    ///
    /// This method parses and evaluates the provided PowerShell script,
    /// handling errors gracefully and returning the result as a formatted
    /// string. It's the recommended method for simple script evaluation.
    ///
    /// # Arguments
    ///
    /// * `script` - A string slice containing the PowerShell script to
    ///   evaluate.
    ///
    /// # Returns
    ///
    /// * `Result<String, ParserError>` - The output of the script evaluation,
    ///   or an error if parsing/evaluation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ps_parser::PowerShellSession;
    ///
    /// let mut session = PowerShellSession::new();
    ///
    /// // Simple arithmetic
    /// let result = session.safe_eval("1 + 2 * 3").unwrap();
    /// assert_eq!(result, "7");
    ///
    /// // Variable assignment and retrieval
    /// let result = session.safe_eval("$name = 'World'; \"Hello $name\"").unwrap();
    /// assert_eq!(result, "Hello World");
    /// ```
    pub fn safe_eval(&mut self, script: &str) -> Result<String, ParserError> {
        let script_res = self.parse_input(script)?;
        Ok(script_res.result().to_string())
    }

    pub fn deobfuscate_script(&mut self, script: &str) -> Result<String, ParserError> {
        self.push_scope_session();
        let script_res = self.parse_input(script)?;
        self.pop_scope_session();
        Ok(script_res.deobfuscated().to_string())
    }

    pub fn env_variables(&self) -> HashMap<String, PsValue> {
        self.variables
            .get_env()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }

    pub fn session_variables(&self) -> HashMap<String, PsValue> {
        self.variables
            .get_global()
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect()
    }

    /// Parses and evaluates a PowerShell script, returning detailed results.
    ///
    /// This method provides comprehensive information about the parsing and
    /// evaluation process, including the final result, generated output,
    /// any errors encountered, and the tokenized representation of the
    /// script. It's particularly useful for debugging and deobfuscation.
    ///
    /// # Arguments
    ///
    /// * `input` - A string slice containing the PowerShell script to parse and
    ///   evaluate.
    ///
    /// # Returns
    ///
    /// * `Result<ScriptResult, ParserError>` - A detailed result containing the
    ///   evaluation outcome, output, errors, and tokens, or a parsing error if
    ///   the script is malformed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ps_parser::PowerShellSession;
    ///
    /// let mut session = PowerShellSession::new();
    /// let script_result = session.parse_input("$a = 42; Write-Output $a").unwrap();
    ///
    /// println!("Final result: {:?}", script_result.result());
    /// println!("Generated output: {:?}", script_result.output());
    /// println!("Parsing errors: {:?}", script_result.errors());
    /// println!("Deobfuscated code: {:?}", script_result.deobfuscated());
    /// ```
    pub fn parse_input(&mut self, input: &str) -> Result<ScriptResult, ParserError> {
        self.variables.init();
        let (script_last_output, mut result) = self.parse_subscript(input)?;
        self.variables.clear_script_functions();
        Ok(ScriptResult::new(
            script_last_output,
            std::mem::take(&mut result.output),
            std::mem::take(&mut result.deobfuscated),
            std::mem::take(&mut self.tokens),
            std::mem::take(&mut self.errors),
            self.variables
                .script_scope()
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        ))
    }

    pub(crate) fn parse_subscript(&mut self, input: &str) -> Result<(Val, Results), ParserError> {
        let mut pairs = PowerShellSession::parse(Rule::program, input)?;
        //create new scope for script
        self.results.push(Results::new());

        let program_token = pairs.next().expect("");

        let mut script_last_output = Val::default();

        if let Rule::program = program_token.as_rule() {
            let pairs = program_token.into_inner();

            for token in pairs {
                let token_str = token.as_str();
                match token.as_rule() {
                    Rule::statement_terminator => continue,
                    Rule::EOI => break,
                    _ => {}
                };

                let result = self.eval_statement(token.clone());
                self.variables.set_status(result.is_ok());

                if let Ok(Val::NonDisplayed(_)) = &result {
                    continue;
                }

                script_last_output = match result {
                    Ok(val) => {
                        if val != Val::Null {
                            self.add_output_statement(val.display().into());
                            self.add_deobfuscated_statement(val.cast_to_script());
                        }

                        val
                    }
                    Err(e) => {
                        self.errors.push(e);
                        self.add_deobfuscated_statement(token_str.into());
                        Val::Null
                    }
                };
            }
        }

        Ok((script_last_output, self.results.pop().unwrap_or_default()))
    }

    fn add_function(
        &mut self,
        name: String,
        func: ScriptBlock,
        scope: Option<Scope>,
    ) -> ParserResult<Val> {
        // let func_str= func.to_function(&name, &scope);
        // self.add_deobfuscated_statement(func_str);

        if let Some(Scope::Global) = &scope {
            self.variables.add_global_function(name.clone(), func);
        } else {
            self.variables.add_script_function(name.clone(), func);
        }

        Err(ParserError::Skip)
    }

    pub(crate) fn parse_function_statement(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::function_statement);

        let mut pair = token.into_inner();

        let function_keyword_token = pair.next().unwrap();
        check_rule!(function_keyword_token, Rule::function_keyword);

        let mut next_token = pair.next().unwrap();
        let scope = if next_token.as_rule() == Rule::scope_keyword {
            let scope = Scope::from(next_token.as_str());
            next_token = pair.next().unwrap();
            Some(scope)
        } else {
            None
        };

        let function_name_token = next_token;
        check_rule!(function_name_token, Rule::function_name);
        let fname = function_name_token.as_str().to_ascii_lowercase();

        let Some(mut next_token) = pair.next() else {
            //empty function
            return self.add_function(fname, ScriptBlock::empty(), scope);
        };

        let params = if next_token.as_rule() == Rule::parameter_list {
            let param_list = self.parse_parameter_list(next_token)?;
            if let Some(token) = pair.next() {
                next_token = token;
            } else {
                return self.add_function(fname, ScriptBlock::empty(), scope);
            }

            param_list
        } else {
            Vec::new()
        };
        check_rule!(next_token, Rule::script_block);

        let mut script_block = self.parse_script_block(next_token)?;

        if script_block.params.0.is_empty() {
            script_block = script_block.with_params(params);
        }

        self.add_function(fname, script_block, scope)
    }

    pub(crate) fn eval_if_statement(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::if_statement);
        let mut pair = token.into_inner();
        let condition_token = pair.next().unwrap();
        let true_token = pair.next().unwrap();
        let condition_val = self.eval_pipeline(condition_token.clone())?;
        let res = if condition_val.cast_to_bool() {
            self.eval_statement_block(true_token)?
        } else if let Some(mut token) = pair.next() {
            if token.as_rule() == Rule::elseif_clauses {
                for else_if in token.into_inner() {
                    let mut pairs = else_if.into_inner();
                    let condition_token = pairs.next().unwrap();
                    let statement_token = pairs.next().unwrap();
                    let condition_val = self.eval_pipeline(condition_token)?;
                    if condition_val.cast_to_bool() {
                        return self.eval_statement_block(statement_token);
                    }
                }
                let Some(token2) = pair.next() else {
                    return Ok(Val::Null);
                };
                token = token2;
            }
            if token.as_rule() == Rule::else_condition {
                let statement_token = token.into_inner().next().unwrap();
                self.eval_statement_block(statement_token)?
            } else {
                Val::Null
            }
        } else {
            Val::Null
        };

        Ok(res)
    }

    fn eval_flow_control_statement(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::flow_control_statement);
        let token = token.into_inner().next().unwrap();

        Ok(match token.as_rule() {
            Rule::flow_control_label_statement => Val::Null, //TODO
            Rule::flow_control_pipeline_statement => {
                let token = token.into_inner().next().unwrap();
                //todo: throw, return or exit
                if let Some(pipeline_token) = token.into_inner().next() {
                    self.eval_pipeline(pipeline_token)?
                } else {
                    Val::Null
                }
            }
            _ => unexpected_token!(token),
        })
    }

    fn eval_statement(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        match token.as_rule() {
            Rule::pipeline => self.eval_pipeline(token),
            Rule::if_statement => self.eval_if_statement(token),
            Rule::flow_control_statement => self.eval_flow_control_statement(token),
            Rule::function_statement => self.parse_function_statement(token),
            Rule::statement_terminator => Ok(Val::Null),
            Rule::EOI => Ok(Val::Null),
            _ => {
                not_implemented!(token)
            }
        }
    }

    fn safe_eval_sub_expr(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        // match self.eval_statements(token.clone()) {
        //     Ok(vals) => Ok(Val::Array(vals)),
        //     Err(err) => {
        //         self.errors.push(err);
        //         Ok(Val::ScriptText(token.as_str().to_string()))
        //     }
        // }
        check_rule!(token, Rule::sub_expression);
        let Some(inner_token) = token.into_inner().next() else {
            return Ok(Val::Null);
        };
        let mut inner_val = self.eval_pipeline(inner_token)?;
        if let Val::ScriptText(script) = &mut inner_val {
            *script = format!("$({})", script);
            //self.tokens.push(Token::SubExpression(script.clone()));
        }
        Ok(inner_val)
    }

    fn eval_statement_block(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        Ok(self
            .safe_eval_statements(token)?
            .iter()
            .last()
            .cloned()
            .unwrap_or(Val::Null))
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

    fn safe_eval_statements(&mut self, token: Pair<'a>) -> ParserResult<Vec<Val>> {
        //check_rule!(token, Rule::statements);
        let pairs = token.into_inner();
        let mut statements = vec![];

        for token in pairs {
            match self.eval_statement(token.clone()) {
                Ok(s) => statements.push(s),
                Err(err) => {
                    self.errors.push(err);
                    statements.push(Val::ScriptText(token.as_str().to_string()));
                }
            }
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
                Rule::sub_expression => self.safe_eval_sub_expr(token)?.cast_to_string(),
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
                let token_string = token.as_str().to_string();
                let stripped_prefix = token_string
                    .strip_prefix("'")
                    .unwrap_or(token_string.as_str());
                let stripped_suffix = stripped_prefix.strip_suffix("'").unwrap_or(stripped_prefix);
                stripped_suffix.to_string()
            }
            Rule::singlequoted_multiline_string_literal => {
                let mut res_str = String::new();
                let pairs = token.into_inner();
                for token in pairs {
                    res_str.push_str(token.as_str());
                }
                res_str
            }
            _ => unexpected_token!(token),
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
            Rule::special_variable => {
                VarName::new_with_scope(Scope::Special, token.as_str().to_string())
            }
            Rule::parenthesized_variable => {
                Self::parse_variable(token.into_inner().next().unwrap())?
            }
            Rule::braced_variable => {
                let token = token.into_inner().next().unwrap();
                let var = token.as_str().to_ascii_lowercase();
                let splits: Vec<&str> = var.split(":").collect();
                if splits.len() == 2 {
                    VarName::new_with_scope(Scope::from(splits[0]), splits[1].to_string())
                } else {
                    VarName::new(None, var)
                }
            }
            Rule::scoped_variable => {
                let mut pairs = token.into_inner();
                let mut token = pairs.next().unwrap();

                let scope = if token.as_rule() == Rule::scope_keyword {
                    let scope = token.as_str().to_ascii_lowercase();
                    token = pairs.next().unwrap();
                    check_rule!(token, Rule::var_name);
                    Some(Scope::from(scope.as_str()))
                } else {
                    None
                };
                VarName::new(scope, token.as_str().to_ascii_lowercase())
            }
            _ => unexpected_token!(token),
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
            _ => unexpected_token!(token),
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
                _ => unexpected_token!(token),
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
                _ => unexpected_token!(token),
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
            _ => unexpected_token!(token),
        };

        Ok(res)
    }

    fn eval_type_literal(&mut self, token: Pair<'a>) -> ParserResult<ValType> {
        check_rule!(token, Rule::type_literal);

        let token = token.into_inner().next().unwrap();
        check_rule!(token, Rule::type_spec);
        Ok(ValType::cast(token.as_str())?)
    }

    fn parse_script_block(&mut self, token: Pair<'a>) -> ParserResult<ScriptBlock> {
        check_rule!(token, Rule::script_block);

        let raw_text = token.as_str().to_string();

        let mut pairs = token.into_inner();
        let Some(mut token) = pairs.next() else {
            return Ok(ScriptBlock::new(vec![], String::new(), raw_text));
        };
        //let mut token = pairs.next().unwrap();

        let (params, _params_str) = if token.as_rule() == Rule::param_block {
            let params = self.parse_param_block(token.clone())?;
            let params_str = token.as_str().to_string();
            token = pairs.next().unwrap();
            (params, params_str)
        } else {
            (vec![], String::new())
        };

        check_rule!(token, Rule::script_block_body);
        let script_body = token.as_str().to_string();

        Ok(ScriptBlock::new(params, script_body, raw_text))

        //todo is it necessary?
        // Ok(if let Ok(deobfuscated_body) =
        // self.deobfuscate_script(&script_body) {
        //     ScriptBlock::new(params, deobfuscated_body.clone(),
        // format!("{};{}", params_str, deobfuscated_body)) } else {
        //     ScriptBlock::new(params, script_body, raw_text)
        // })
    }

    fn parse_script_block_expression(&mut self, token: Pair<'a>) -> ParserResult<ScriptBlock> {
        check_rule!(token, Rule::script_block_expression);
        let mut pairs = token.into_inner();
        self.parse_script_block(pairs.next().unwrap())
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
            _ => unexpected_token!(key_token),
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
            Rule::script_block_expression => {
                Val::ScriptBlock(self.parse_script_block_expression(token)?)
            }
            Rule::hash_literal_expression => self.eval_hash_literal(token)?,
            Rule::string_literal => self.eval_string_literal(token)?,
            Rule::number_literal => self.eval_number_literal(token)?,
            Rule::type_literal => Val::init(self.eval_type_literal(token)?)?,
            Rule::variable => self.get_variable(token)?,
            _ => unexpected_token!(token),
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
            Rule::float => {
                let float_str = token.as_str().trim();
                Val::Float(float_str.parse::<f64>()?)
                //todo: handle all border cases
            }
            _ => unexpected_token!(token),
        };
        Ok(v)
    }

    fn eval_unary_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::unary_exp);
        let token = token.into_inner().next().unwrap();
        match token.as_rule() {
            Rule::expression_with_unary_operator => self.eval_expression_with_unary_operator(token),
            Rule::primary_expression => self.eval_primary_expression(token),
            _ => unexpected_token!(token),
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
        fn range(mut left: i64, right: i64) -> Vec<Val> {
            let mut v = Vec::new();
            if left <= right {
                loop {
                    v.push(left);
                    if left == right {
                        break;
                    }
                    left += 1;
                }
            } else {
                loop {
                    v.push(left);
                    if left == right {
                        break;
                    }
                    left -= 1;
                }
            }
            v.into_iter().map(Val::Int).collect()
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
            _ => unexpected_token!(token),
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
                log::error!("No arithmetic function for operator: {}", op.as_str());
                return Err(ParserError::NotImplemented(format!(
                    "No arithmetic function for operator: {}",
                    op.as_str()
                )));
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
                log::error!("No arithmetic function for operator: {}", op.as_str());
                return Err(ParserError::NotImplemented(format!(
                    "No arithmetic function for operator: {}",
                    op.as_str()
                )));
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_mult(mult)?;
            res = fun(res, right_op)?;
        }

        Ok(res)
    }

    fn eval_split_special_case(
        &mut self,
        script_block: ScriptBlock,
        input: Val,
    ) -> ParserResult<Vec<String>> {
        let mut res_vec = vec![];
        let mut parts = String::new();
        let input_str = input.cast_to_string();
        let characters = input_str.chars();

        // filtered_elements.join("")
        for ch in characters {
            let b = match script_block.run(vec![], self, Some(Val::String(ch.to_string().into()))) {
                Err(er) => {
                    self.errors.push(er);
                    false
                }
                Ok(res) => res.val.cast_to_bool(),
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
                log::error!("No string predicate for operator: {}", op.as_str());
                return Err(ParserError::NotImplemented(format!(
                    "No string predicate for operator: {}",
                    op.as_str()
                )));
            };

            let token = pairs.next().unwrap();
            let right_op = match token.as_rule() {
                Rule::script_block_expression => {
                    let script_block = self.parse_script_block_expression(token)?;

                    return Ok(Val::Array(
                        self.eval_split_special_case(script_block, res)?
                            .into_iter()
                            .map(|s| Val::String(s.into()))
                            .collect::<Vec<_>>(),
                    ));
                }
                Rule::additive_exp => self.eval_additive(token)?,
                _ => unexpected_token!(token),
            };
            log::trace!("res: {:?}, right_op: {:?}", &res, &right_op);
            res = fun(res, right_op)?;
            log::trace!("res: {:?}", &res);
        }

        Ok(res)
    }

    fn parse_param_block(&mut self, token: Pair<'a>) -> ParserResult<Vec<Param>> {
        check_rule!(token, Rule::param_block);
        let mut pairs = token.into_inner();
        let Some(token) = pairs.next() else {
            return Ok(vec![]);
        };

        self.parse_parameter_list(token)
    }

    fn parse_parameter_list(&mut self, token: Pair<'a>) -> ParserResult<Vec<Param>> {
        check_rule!(token, Rule::parameter_list);
        let mut params = vec![];
        let param_list_pairs = token.into_inner();
        for script_parameter_token in param_list_pairs {
            check_rule!(script_parameter_token, Rule::script_parameter);
            params.push(self.parse_script_parameter(script_parameter_token)?);
        }
        Ok(params)
    }

    fn parse_attribute_list(&mut self, token: Pair<'a>) -> ParserResult<Option<ValType>> {
        check_rule!(token, Rule::attribute_list);
        let attribute_list_pairs = token.into_inner();
        for attribute_token in attribute_list_pairs {
            check_rule!(attribute_token, Rule::attribute);
            let attribute_type_token = attribute_token.into_inner().next().unwrap();
            match attribute_type_token.as_rule() {
                Rule::attribute_info => {
                    //skip for now
                    continue;
                }
                Rule::type_literal => {
                    return Ok(Some(self.eval_type_literal(attribute_type_token)?));
                }
                _ => unexpected_token!(attribute_type_token),
            }
        }
        Ok(None)
    }
    fn parse_script_parameter(&mut self, token: Pair<'a>) -> ParserResult<Param> {
        check_rule!(token, Rule::script_parameter);
        let mut pairs = token.into_inner();
        let mut token = pairs.next().unwrap();

        let type_literal = if token.as_rule() == Rule::attribute_list {
            let type_literal = self.parse_attribute_list(token)?;
            token = pairs.next().unwrap();
            type_literal
        } else {
            None
        };

        check_rule!(token, Rule::variable);
        let var_name = Self::parse_variable(token)?;

        let default_value = if let Some(default_value_token) = pairs.next() {
            check_rule!(default_value_token, Rule::script_parameter_default);
            let default_value_expr = default_value_token.into_inner().next().unwrap();
            let default_value = self.eval_value(default_value_expr)?;
            Some(default_value)
        } else {
            None
        };
        Ok(Param::new(type_literal, var_name.name, default_value))
    }

    fn eval_bitwise_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::bitwise_exp);

        let mut pairs = token.into_inner();
        let mut res = self.eval_comparison_exp(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            check_rule!(op, Rule::bitwise_operator);
            let Some(fun) = BitwisePred::get(op.as_str()) else {
                log::error!("No bitwise predicate for operator: {}", op.as_str());
                return Err(ParserError::NotImplemented(format!(
                    "No bitwise predicate for operator: {}",
                    op.as_str()
                )));
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_comparison_exp(mult)?;
            res = fun(res, right_op);
        }

        Ok(res)
    }

    fn parse_cmdlet_command_name(&mut self, token: Pair<'a>) -> ParserResult<Command> {
        check_rule!(token, Rule::cmdlet_command);

        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();
        let command_name = match token.as_rule() {
            Rule::command_name => token.as_str(),
            Rule::where_command_name => "where-object",
            Rule::foreach_command_name => "foreach-object",
            Rule::powershell_command_name => "powershell",
            _ => unexpected_token!(token),
        };

        let mut command = Command::cmdlet(command_name);
        if Rule::command_name == token.as_rule() {
            command.set_session_scope(SessionScope::New);
        }
        Ok(command)
    }

    fn parse_command_args(&mut self, pairs: Pairs<'a>) -> ParserResult<Vec<CommandElem>> {
        let mut args = vec![];
        for command_element_token in pairs {
            let token_string = command_element_token.as_str().to_string();
            match command_element_token.as_rule() {
                Rule::command_argument => {
                    let arg_token = command_element_token.into_inner().next().unwrap();
                    let arg = match arg_token.as_rule() {
                        Rule::array_literal_exp => self.eval_array_literal_exp(arg_token)?,
                        Rule::script_block_expression => {
                            Val::ScriptBlock(self.parse_script_block_expression(arg_token)?)
                        }
                        Rule::parenthesized_expression => {
                            let token = arg_token.into_inner().next().unwrap();
                            self.eval_pipeline(token)?
                        }
                        _ => Val::ScriptText(arg_token.as_str().to_string()),
                    };
                    args.push(CommandElem::Argument(arg));
                }
                Rule::command_parameter => {
                    args.push(CommandElem::Parameter(token_string.to_ascii_lowercase()))
                }
                Rule::argument_list => args.push(CommandElem::ArgList(token_string)),
                Rule::redirection => { //todo: implement redirection
                }
                Rule::stop_parsing => { //todo: stop parsing
                }
                _ => unexpected_token!(command_element_token),
            }
        }
        Ok(args)
    }

    fn eval_command(&mut self, token: Pair<'a>, piped_arg: Option<Val>) -> ParserResult<Val> {
        check_rule!(token, Rule::command);
        let mut pairs = token.into_inner();

        let command_token = pairs.next().unwrap();
        let mut command = match command_token.as_rule() {
            Rule::cmdlet_command => self.parse_cmdlet_command_name(command_token)?,
            Rule::invocation_command => self.parse_invocation_command(command_token)?,
            _ => unexpected_token!(command_token),
        };

        let mut args = self.parse_command_args(pairs)?;
        if let Some(arg) = piped_arg {
            args.insert(0, CommandElem::Argument(arg));
        }

        command.with_args(args);
        match command.execute(self) {
            Ok(CommandOutput {
                val,
                deobfuscated: _deobfuscated,
            }) => Ok(val),
            Err(e) => {
                self.errors.push(e);
                Ok(Val::ScriptText(command.to_string()))
            }
        }

        // if let Some(msg) = deobfuscated {
        //     self.add_deobfuscated_statement(msg);
        // }
    }

    fn add_deobfuscated_statement(&mut self, msg: String) {
        if let Some(last) = self.results.last_mut() {
            last.deobfuscated.push(msg);
        }
    }

    fn add_output_statement(&mut self, msg: StreamMessage) {
        if let Some(last) = self.results.last_mut() {
            last.output.push(msg);
        }
    }

    fn parse_invocation_command(&mut self, token: Pair<'a>) -> ParserResult<Command> {
        check_rule!(token, Rule::invocation_command);

        let invocation_command_token = token.into_inner().next().unwrap();

        let mut session_scope = match invocation_command_token.as_rule() {
            Rule::current_scope_invocation_command => SessionScope::Current,
            Rule::new_scope_invocation_command => SessionScope::New,
            _ => unexpected_token!(invocation_command_token),
        };

        let token_inner = invocation_command_token.into_inner().next().unwrap();

        let mut command = match token_inner.as_rule() {
            Rule::cmdlet_command => {
                session_scope = SessionScope::New;
                self.parse_cmdlet_command_name(token_inner)?
            }
            Rule::primary_expression => {
                let primary = self.eval_primary_expression(token_inner)?;
                if let Val::ScriptBlock(script_block) = primary {
                    Command::script_block(script_block)
                } else {
                    Command::cmdlet(&primary.cast_to_script())
                }
            }
            Rule::path_command_name => Command::path(token_inner.as_str()),
            _ => unexpected_token!(token_inner),
        };

        command.set_session_scope(session_scope);
        Ok(command)
    }

    fn eval_redirected_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::redirected_expression);

        let expression_token = token.into_inner().next().unwrap();
        //todo: handle redirections

        self.eval_expression(expression_token)
    }

    fn eval_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::expression);
        let token_string = token.as_str().trim().to_string();

        let mut pairs = token.into_inner();
        let mut res = self.eval_bitwise_exp(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            check_rule!(op, Rule::logical_operator);
            let Some(fun) = LogicalPred::get(op.as_str()) else {
                log::error!("No logical predicate for operator: {}", op.as_str());
                return Err(ParserError::NotImplemented(format!(
                    "No logical predicate for operator: {}",
                    op.as_str()
                )));
            };

            let mult = pairs.next().unwrap();
            let right_op = self.eval_bitwise_exp(mult)?;
            res = Val::Bool(fun(res, right_op));
        }
        self.tokens
            .push(Token::Expression(token_string, res.clone().into()));

        Ok(res)
    }

    fn eval_pipeline_tail(&mut self, token: Pair<'a>, mut piped_arg: Val) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline_tail);
        let pairs = token.into_inner();

        for token in pairs {
            //self.variables.set_ps_item(arg);
            piped_arg = self.eval_command(token, Some(piped_arg))?;
        }

        Ok(piped_arg)
    }

    fn eval_pipeline_with_tail(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline_with_tail);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        let result: Val = match token.as_rule() {
            Rule::redirected_expression => self.eval_redirected_expression(token)?,
            Rule::command => self.eval_command(token, None)?,
            _ => unexpected_token!(token),
        };

        if let Some(token) = pairs.next() {
            match token.as_rule() {
                Rule::pipeline_tail => Ok(self.eval_pipeline_tail(token, result)?),
                _ => unexpected_token!(token),
            }
        } else {
            Ok(result)
        }
    }

    fn eval_pipeline(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::pipeline);
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();

        match token.as_rule() {
            Rule::assignment_exp => self.eval_assigment_exp(token),
            Rule::pipeline_with_tail => self.eval_pipeline_with_tail(token),
            _ => unexpected_token!(token),
        }
    }

    fn safe_eval_pipeline(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        let res = self.eval_pipeline(token.clone());

        let v = match res {
            Ok(val) => val,
            Err(err) => {
                self.errors.push(err);
                Val::ScriptText(token.as_str().to_string())
            }
        };

        Ok(v)
    }

    fn eval_cast_expression(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::cast_expression);

        let mut pairs = token.into_inner();
        let type_token = pairs.next().unwrap();
        check_rule!(type_token, Rule::type_literal);
        let val_type = self.eval_type_literal(type_token)?;

        let token = pairs.next().unwrap();
        let res = match token.as_rule() {
            Rule::parenthesized_expression => {
                let token = token.into_inner().next().unwrap();
                self.safe_eval_pipeline(token)?
            }
            Rule::range_exp => self.eval_range_exp(token)?,
            Rule::unary_exp => self.eval_unary_exp(token)?,
            _ => unexpected_token!(token),
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
        let right_op = self.eval_statement(right_token.clone())?;

        let Some(pred) = pred else {
            log::error!("No arithmetic function for operator: {}", op.as_str());
            return Err(ParserError::NotImplemented(format!(
                "No arithmetic function for operator: {}",
                op.as_str()
            )));
        };
        let op_result = pred(var, right_op)?;
        self.variables.set(&var_name, op_result.clone())?;

        //we want save each assignment statement
        self.add_deobfuscated_statement(format!("{} = {}", var_name, op_result.cast_to_script()));

        Ok(Val::NonDisplayed(Box::new(op_result)))
    }

    fn push_scope_session(&mut self) {
        self.variables.push_scope_session();
    }

    fn pop_scope_session(&mut self) {
        self.variables.pop_scope_session();
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
