mod command;
mod comparison;
mod predicates;
mod value;
mod variables;

use command::PsCommand;
use comparison::Comparison;
use pest::Parser;
use pest_derive::Parser;
use predicates::Predicates;
pub use value::{Val, ValType};
use variables::Variables;
use thiserror_no_std::Error;

type PestError = pest::error::Error<Rule>;
type Pair<'i> = ::pest::iterators::Pair<'i, Rule>;

use value::ValError;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("PestError: {0}")]
    PestError(PestError),

    #[error("ValError: {0}")]
    ValError(ValError),
}

impl From<PestError> for ParserError{
    fn from(value: PestError) -> Self {
        Self::PestError(value)
    }
}

impl From<ValError> for ParserError{
    fn from(value: ValError) -> Self {
        Self::ValError(value)
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

    pub fn evaluate(&mut self, input: &str) -> ParserResult<String> {
        let mut pairs = PowerShellParser::parse(Rule::program, input)?;
        let program_token = pairs.next().expect("");

        if let Rule::program = program_token.as_rule() {
            let pairs = program_token.into_inner();
            for token in pairs {
                //self.parse_statement(pair)?;
                match token.as_rule() {
                    Rule::expression => {
                        //println!("Assignment: {}", token.as_str());
                        self.eval_expression(token)?;
                    }
                    Rule::EOI => {
                        return Ok(String::new());
                    }
                    _ => {
                        println!("not implemented: {:?}", token.as_rule());
                    }
                }
            }
        }

        Ok(String::new())
    }

    fn eval_num(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();
        let v = match token.as_rule() {
            Rule::int => token.as_str().parse::<i64>().unwrap(),
            Rule::binary => {
                todo!()
            }
            Rule::hex => {
                let lowercase = token.as_str().to_ascii_lowercase();
                i64::from_str_radix(lowercase.strip_prefix("0x").unwrap(), 16).unwrap()
            }
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
            }
        };
        Ok(Val::Int(v))
    }

    fn eval_cast_exp(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::cast_expression);
        let mut tokens = token.into_inner();

        let type_name_token = tokens.next().expect("Failed to get token");
        check_rule!(type_name_token, Rule::type_name);

        let val_type = ValType::cast(type_name_token.as_str())?;

        let expression = tokens.next().expect("Failed to get token");
        let mut val = match expression.as_rule() {
            Rule::expression => self.eval_expression(expression)?,
            Rule::number => self.eval_num(expression)?,
            Rule::postfix_expr => self.eval_postfix(expression)?,
            _ => {
                println!("token_rule: {:?}", expression.as_rule());
                todo!()
            }
        };
        Ok(val.cast(val_type)?)
    }

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

    fn eval_atomic(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        //check_rule!(token, Rule::atomic);
        let res = match token.as_rule() {
            Rule::cast_expression => self.eval_cast_exp(token)?,
            Rule::number => self.eval_num(token)?,
            Rule::string_literal => {
                //let mut res_string = String::new();
                if let Some(stripped_prefix) = token.as_str().to_string().strip_prefix("'") {
                    if let Some(stripped_suffix) = stripped_prefix.to_string().strip_suffix("'") {
                        Val::String(stripped_suffix.to_string())
                    } else {
                        panic!("no suffix")
                    }
                } else {
                    panic!("no prefix")
                }
                //Val::String(res_string)
            }
            Rule::expandable_string_content => {
                let x = self.expand_string(token)?;
                if let Val::String(s) = &x {
                    println!("expanded: {s}");
                }
                x
            }
            Rule::expression => self.eval_expression(token)?,
            Rule::postfix_expr => self.eval_postfix(token)?,
            Rule::variable => self.variables.get(token.as_str()),
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
            }
        };

        Ok(res)
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

    fn eval_postfix(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::postfix_expr);

        let mut pairs = token.into_inner();
        let mut res = self.eval_atomic(pairs.next().unwrap())?;
        let mut command = None;
        let mut args = None;

        //eval member_suffix. To fix. It's quite prymitive right now
        while let Some(op) = pairs.next() {
            match op.as_rule() {
                Rule::dot_member => {
                    command = self.eval_command(op);
                    //println!("command {:?}", command);
                }
                Rule::method_invocation => {
                    args = self.eval_args(op);
                }
                _ => {
                    println!("token.rule(): {:?}", op.as_rule());
                    panic!()
                }
            }
        }

        if let (Some(method), Some(args)) = (command, args) {
            if let Some(r) = PsCommand::call(res.clone(), method, args) {
                res = r;
            }
        }

        Ok(res)
    }

    fn eval_mult(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::multiplicative_exp);
        let mut pairs = token.into_inner();
        let mut res = self.eval_atomic(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            let Some(fun) = Predicates::get(op.as_str()) else {
                panic!()
            };

            let postfix = pairs.next().unwrap();
            let right_op = self.eval_postfix(postfix)?;
            res = fun(res, right_op);
        }

        Ok(res)
    }

    fn eval_additive(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::additive_exp);

        let mut pairs = token.into_inner();
        let mut res = self.eval_mult(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            let Some(fun) = Predicates::get(op.as_str()) else {
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
        let first_operand = pairs.next().unwrap();
        check_rule!(first_operand, Rule::additive_exp);
        let v1 = self.eval_additive(first_operand)?;

        let operator_token = pairs.next().unwrap();
        check_rule!(operator_token, Rule::comparison_op);

        let second_operand = pairs.next().unwrap();
        check_rule!(second_operand, Rule::additive_exp);
        let v2 = self.eval_additive(second_operand)?;

        let token = operator_token.into_inner().next().unwrap();
        Ok(match token.as_rule() {
            Rule::replace_op => {
                let Some(replace_fn) = Comparison::replace_op(token.as_str()) else {
                    panic!();
                };
                replace_fn(v1, vec![v2])
            }
            Rule::cmp_op => {
                let Some(cmp_fn) = Comparison::cmp_op(token.as_str()) else {
                    panic!();
                };
                Val::Bool(cmp_fn(v1, v2))
            }
            _ => todo!()
        })
    }

    fn eval_expression(&mut self, token_exp: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token_exp, Rule::expression);

        let mut res = Val::default();
        let pairs = token_exp.clone().into_inner();
        for token in pairs {
            match token.as_rule() {
                Rule::additive_exp => {
                    res.add(self.eval_additive(token)?)?;
                }
                Rule::multiplicative_exp => {
                    res.add(self.eval_mult(token)?)?;
                }
                Rule::comparison_exp => {
                    res.add(self.eval_comparison_exp(token)?)?;
                }
                Rule::command_call => {
                    res.add(self.eval_command_call(token)?)?;
                }
                Rule::assignment_exp => {
                    self.eval_assigment_exp(token)?;
                }
                _ => {
                    println!("eval_expression not implemented: {:?}", token.as_rule());
                }
            }
        }
        //println!("exp: {}, res: {:?}", token_exp.as_str(), res);
        Ok(res)
    }

    fn expand_string(&mut self, token: Pair<'a>) -> ParserResult<Val> {
        check_rule!(token, Rule::expandable_string_content);
        let mut res = Val::default();
        let pairs = token.into_inner();
        for token in pairs {
            match token.as_rule() {
                Rule::string_text => {
                    res.add(Val::String(token.as_str().to_string()))?;
                }
                Rule::expression => {
                    res.add(self.eval_expression(token)?)?;
                }
                _ => {
                    println!("expand_string not implemented: {:?}", token.as_rule());
                }
            }
        }

        Ok(res)
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
        let var = self.variables.get(variable_token.as_str());

        let assignement_op = pairs.next().unwrap();

        //get operand
        let op = assignement_op.into_inner().next().unwrap();
        let pred = Predicates::get(op.as_str());

        let expression_token = pairs.next().unwrap();
        let expression_result = self.eval_expression(expression_token)?;

        let Some(pred) = pred else { panic!() };

        self.variables
            .set(variable_token.as_str(), pred(var, expression_result));
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
    fn amsi_fail() {
        let input = r#"
#Matt Graebers second Reflection method 
$VMRviwsbtehQfPtxbt=$null;
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Âmsí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))"

"#;

        let _ = PowerShellParser::parse(Rule::program, input).unwrap();
    }
}
