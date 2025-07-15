use std::collections::HashMap;

use pest::Parser;
use pest_derive::Parser;

type PestError = pest::error::Error<Rule>;
type PestResult<T> = core::result::Result<T, PestError>;
type Pairs<'i> = ::pest::iterators::Pairs<'i, Rule>;
type Pair<'i> = ::pest::iterators::Pair<'i, Rule>;

use std::sync::LazyLock;

macro_rules! check_rule {
    ($pair:expr, $rule:pat) => {
        if !matches!($pair.as_rule(), $rule) {
            println!("rule: {:?}", $pair.as_rule());
            panic!()
        }
    };
}

fn add(arg1: Val, arg2: Val) -> Val {
    arg1.add(arg2)
}

fn sub(arg1: Val, arg2: Val) -> Val {
    match (arg1, arg2) {
        (Val::Int(i1), Val::Int(i2)) => Val::Int(i1-i2),
        _ => panic!(),
    }
}

fn mul(arg1: Val, arg2: Val) -> Val {
    match (arg1, arg2) {
        (Val::Int(i1), Val::Int(i2)) => Val::Int(i1*i2),
        _ => panic!(),
    }
}

fn div(arg1: Val, arg2: Val) -> Val {
    match (arg1, arg2) {
        (Val::Int(i1), Val::Int(i2)) => Val::Int(i1/i2),
        _ => panic!(),
    }
}

fn modulo(arg1: Val, arg2: Val) -> Val {
    match (arg1, arg2) {
        (Val::Int(i1), Val::Int(i2)) => Val::Int(i1%i2),
        _ => panic!(),
    }
}

type PRED_TYPE = Box<dyn Fn(Val, Val) -> Val>;

const PRED_MAP: LazyLock<HashMap<String, PRED_TYPE>> = LazyLock::new(|| {
    HashMap::from([
        ("+".to_string(), Box::new(add) as PRED_TYPE),
        ("-".to_string(), Box::new(sub) as PRED_TYPE),
        ("*".to_string(), Box::new(mul) as PRED_TYPE),
        ("/".to_string(), Box::new(div) as PRED_TYPE),
        ("%".to_string(), Box::new(modulo) as PRED_TYPE),
    ])
});

#[derive(Parser)]
#[grammar = "powershell.pest"]
pub(crate) struct PowerShellParser;

struct Method {
    field: Val,
    name: String,
    args: Vec<Val>,
}

fn normalize(input: &str, form: &str) -> Option<Val> {
    use unicode_normalization::UnicodeNormalization;

    println!("met:  {} {}", input, form);
    let res = match form {
        "FormD" => input.nfd().filter(|c| c.is_ascii()).collect(),  // Canonical Decomposition
        "FormC" => input.nfc().collect(),  // Canonical Composition
        "FormKD" => input.nfkd().collect(), // Compatibility Decomposition
        "FormKC" => input.nfkc().collect(), // Compatibility Composition
        _ => input.to_string(), // Default: no normalization
    };
    println!("res:  {}", res);
    Some(Val::String(res))
}

impl Method {
    pub fn call(field_name: Val, method_name: Val, args: Vec<Val>) -> Option<Val> {
        println!("elo");
        let Val::String(method) = method_name  else {
            return None;
        };

        let method = method.to_ascii_lowercase();

        match method.as_str() {
            "normalize" => {
                let Val::String(field) = field_name else {
                    return None;
                };
                let Val::String(form) = args[0].clone() else {
                    return None;
                };
                normalize(field.as_str(), form.as_str())
            }
            _ => todo!(),
        }
    }
}


#[derive(Clone, Debug)]
enum Val {
    None,
    NoneChar,
    Int(i64),
    Char(u32),
    String(String),
}

impl Val {
    pub fn add(&self, val: Val) -> Val {
        //println!("self: {:?}, val: {:?}", self, val);
        let res = match (self, &val) {
            (Val::None, _) => val.clone(),
            (Val::Int(i), Val::Int(i2)) => Val::Int(i+i2),
            (Val::NoneChar, Val::Int(i)) => Val::Char(*i as _),
            (Val::NoneChar, Val::Char(c2)) => val,
            (Val::Char(c), Val::Char(c2)) => {
                let mut s = unsafe {char::from_u32_unchecked(*c).to_string()};
                s.push(unsafe {char::from_u32_unchecked(*c2)});
                Val::String(s)
            }
            (Val::Char(c), Val::Int(c2)) => Val::Char(c+*c2 as u32),
            (Val::String(s), Val::Char(c)) => {
                let mut new_s = s.clone();
                new_s.push(unsafe {char::from_u32_unchecked(*c)});
                Val::String(new_s)
            },
            (Val::String(s), Val::String(s2)) => {
                let mut new_s = s.clone();
                new_s.push_str(s2.as_str());
                Val::String(new_s)
            }
            (_, Val::None) => self.clone(),
            _ => {
                panic!()
            },
        };
        //println!("res: {res:?}");
        res
    }

    pub fn sub(&self, val: Val) -> Val {
        //println!("self: {:?}, val: {:?}", self, val);
        let res = match (self, &val) {
            (Val::Int(i), Val::Int(i2)) => Val::Int(i-i2),
            (Val::Char(c), Val::Int(c2)) => Val::Char(c-*c2 as u32),
            _ => {
                panic!()
            },
        };
        println!("res: {res:?}");
        res
    }

    fn from_type(s: &str) -> Self {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "char" => Val::NoneChar,
            "byte" => Val::NoneChar,
            _ => todo!(),
        }
    }

    fn to_string(&self) -> String{
        match self {
            Val::Int(i) => format!("{i}"),
            Val::Char(c) => format!("{c}"),
            Val::String(s) => format!("{s}"),
            _ => todo!(),
        }
    }
}

impl<'a> PowerShellParser {
    fn eval_num(token: Pair<'a>) -> PestResult<Val> {
        let mut pairs = token.into_inner();
        let token = pairs.next().unwrap();
        let v = match token.as_rule() {
            Rule::int => {
                token.as_str().parse::<i64>().unwrap()
            }
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

    pub fn eval_cast_exp(token: Pair<'a>) -> PestResult<Val> {
        let mut tokens = token.into_inner();
        let type_name = tokens.next().expect("Failed to get token");
        if type_name.as_rule() != Rule::type_name {
            panic!()
        }

        let mut val = Val::from_type(type_name.as_str());

        let expression  = tokens.next().expect("Failed to get token");
        match expression.as_rule() {
            Rule::expression => {
                val = val.add(Self::eval_exp(expression)?)
            }
            Rule::number => {
                val = val.add(Self::eval_num(expression)?)
            }
            Rule::postfix_expr => {
                val = val.add(Self::eval_postfix(expression)?)
            }
            _ => {
                println!("token_rule: {:?}", expression.as_rule());
                todo!()
            }
        }

        Ok(val)
    }

    pub fn eval_args(token: Pair<'a>) -> Option<Vec<Val>> {
        check_rule!(token, Rule::method_invocation);
        let token = token.into_inner().next().unwrap();
        check_rule!(token, Rule::argument_list);

        println!("argument_list {:?}", token.as_rule());
        println!("argument_list {:?}", token.as_str());

        let mut args = vec![];
        for token in token.into_inner() {
            match token.as_rule() {
                Rule::expression  => {
                    if let Ok(arg) = Self::eval_exp(token) {
                        args.push(arg);
                    }
                }
                _ => {
                    println!("eval_exp not implemented: {:?}", token.as_rule());
                }
            }
        }
        println!("args {:?}", args);
        Some(args)
    }

    pub fn eval_method(token: Pair<'a>) -> Option<Val> {
        check_rule!(token, Rule::dot_member);
        let token = token.into_inner().next().unwrap();
        println!("dot_member {:?}", token.as_rule());
        println!("dot_member {:?}", token.as_str());
        Self::eval_exp(token).ok()
    }

    pub fn eval_atomic(token: Pair<'a>) -> PestResult<Val> {
        //check_rule!(token, Rule::multiplicative_exp);
        let mut res = Val::None;
        match token.as_rule() {
            Rule::cast_expression => {
                res = res.add(Self::eval_cast_exp(token)?)
            }
            Rule::number => {
                res = res.add(Self::eval_num(token)?)
            }
            Rule::string_literal => {
                let s = token.as_str().to_string().strip_prefix("'").unwrap().to_string().strip_suffix("'").unwrap().to_string();
                res = res.add(Val::String(s))
            }
            Rule::expression  => {
                res = res.add(Self::eval_exp(token)?)
            }
            Rule::postfix_expr   => {
                res = res.add(Self::eval_postfix(token)?)
            }
            _ => {
                println!("token.rule(): {:?}", token.as_rule());
                panic!()
            }
        }
        Ok(res)
    }

    pub fn eval_command_call(token: Pair<'a>) -> PestResult<Val> {
        check_rule!(token, Rule::command_call);
        let mut pairs = token.into_inner();
        let ident = pairs.next().unwrap();

        let mut res = Val::None;
        match ident.as_rule() {
            Rule::command_ident => {
                res = res.add(Val::String(ident.as_str().to_string()))
            }
            _ => {
                println!("eval_command_call token.rule(): {:?}", ident.as_rule());
                panic!()
            }
        }

        while let Some(exp) = pairs.next() {
            todo!()
        }
        Ok(res)
    }

    pub fn eval_postfix(token: Pair<'a>) -> PestResult<Val> {
        check_rule!(token, Rule::postfix_expr);

        let mut pairs = token.into_inner();
        let mut res = Self::eval_atomic(pairs.next().unwrap())?;
        let mut method = None;
        let mut args = None;
        while let Some(op) = pairs.next() {
            match op.as_rule() {
                Rule::dot_member => {
                    method = Self::eval_method(op);
                    println!("method {:?}", method);
                }
                Rule::method_invocation => {
                    args = Self::eval_args(op);
                }
                _ => {
                    println!("token.rule(): {:?}", op.as_rule());
                    panic!()
                }
            }
        }

        if let (Some(method), Some(args)) = (method, args) {
            if let Some(r) = Method::call(res.clone(), method, args) {
                res = r;
            }
        }

        Ok(res)
    }

    pub fn eval_mult(token: Pair<'a>) -> PestResult<Val> {
        check_rule!(token, Rule::multiplicative_exp);
        let mut pairs = token.into_inner();
        let mut res = Self::eval_atomic(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            let postfix = pairs.next().unwrap();
            res = PRED_MAP.get(op.as_str()).unwrap()(res, Self::eval_postfix(postfix)?);
        }

        Ok(res)
    }

    pub fn eval_additive(token: Pair<'a>) -> PestResult<Val> {
        if token.as_rule() != Rule::additive_exp {
            println!("token.rule(): {:?}", token.as_rule());
            panic!("something wrongsdfsd");
        }

        let mut pairs = token.into_inner();
        let mut res = Self::eval_mult(pairs.next().unwrap())?;
        while let Some(op) = pairs.next() {
            let mult = pairs.next().unwrap();
            res = PRED_MAP.get(op.as_str()).unwrap()(res, Self::eval_mult(mult)?);
        }

        Ok(res)
    }

    pub fn eval_comparison_exp(token: Pair<'a>) -> PestResult<Val> {

        fn eval_comp_op(v1: Val, v2: Val, op: &str) -> PestResult<Val> {
            match op {
                "-replace" => {
                    //do nothing write now
                    return Ok(v1);
                }
                _ => panic!(),
            }
            Ok(Val::None)
        }

        check_rule!(token, Rule::comparison_exp);

        let mut pairs = token.into_inner();
        let first_operand = pairs.next().unwrap();
        check_rule!(first_operand, Rule::additive_exp);
        let v1 = Self::eval_additive(first_operand)?;

        let operator_token = pairs.next().unwrap();
        check_rule!(operator_token, Rule::comparison_op);
        let operator = operator_token.as_str();

        let second_operand = pairs.next().unwrap();
        check_rule!(second_operand, Rule::additive_exp);
        let v2 = Self::eval_additive(second_operand)?;

        eval_comp_op(v1,v2,operator)
    }
    pub fn eval_exp(token: Pair<'a>) -> PestResult<Val> {
        if token.as_rule() != Rule::expression {
            println!("token.rule(): {:?}", token.as_rule());
            panic!("something wrongsdfsd");
        }

        let mut res = Val::None;
        let pairs = token.into_inner();
        for token in pairs {
            match token.as_rule() {
                Rule::additive_exp => {
                    res = res.add(Self::eval_additive(token)?);
                }
                Rule::multiplicative_exp  => {
                    res = res.add(Self::eval_mult(token)?);
                }
                Rule::comparison_exp => {
                    res = res.add(Self::eval_comparison_exp(token)?);
                }
                Rule::command_call => {
                    res = res.add(Self::eval_command_call(token)?);
                }
                _ => {
                    println!("eval_exp not implemented: {:?}", token.as_rule());
                }
            }
        }

        Ok(res)
    }

    pub fn expand_string(token: Pair<'a>) -> PestResult<Val> {
        let mut res = Val::None;
        let pairs = token.into_inner();
        for token in pairs {
            match token.as_rule() {
                    Rule::string_text => {
                        res = res.add(Val::String(token.as_str().to_string()));
                    }
                    Rule::expression => {
                        res = res.add(Self::eval_exp(token)?);
                    }
                    _ => {
                        println!("expand_string not implemented: {:?}", token.as_rule());
                    }
                }
        }

        Ok(res)
    }

    pub fn expand_strings(input: &str) -> PestResult<String> {
        let mut pairs = PowerShellParser::parse(Rule::program, input)?;
        let program_token = pairs.next().expect("");

        if let Rule::program = program_token.as_rule() {
            let mut pairs = program_token.into_inner();
            let token = pairs.next().unwrap();

            if let Rule::expression  = token.as_rule() {
                let mut pairs = token.into_inner();
                let token = pairs.next().unwrap();

                if let Rule::assignment_exp  = token.as_rule() {
                    let mut pairs = token.into_inner();
                    let token = pairs.next().unwrap();
                    let token = pairs.next().unwrap();

                    if let Rule::expression  = token.as_rule() {
                        let mut pairs = token.into_inner();
                        let token = pairs.next().unwrap();

                        if let Rule::additive_exp = token.as_rule() {
                            let mut pairs = token.into_inner();
                            let token = pairs.next().unwrap();
                            if let Rule::multiplicative_exp  = token.as_rule() {
                                let mut pairs = token.into_inner();
                                let token = pairs.next().unwrap();

                                if let Rule::postfix_expr  = token.as_rule() {
                                    let mut pairs = token.into_inner();
                                    let token = pairs.next().unwrap();

                                    if let Rule::expandable_string_content  = token.as_rule() {
                                        //println!("Expandable: {}", token.as_str());
                                        return Ok(Self::expand_string(token)?.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        todo!()
    }

    pub fn parse_input(input: &str) -> PestResult<()> {
        let mut pairs = PowerShellParser::parse(Rule::program, input)?;
        let program_token = pairs.next().expect("");

        if let Rule::program = program_token.as_rule() {
            let pairs = program_token.into_inner();
            for token in pairs {
                //Self::parse_statement(pair)?;
                match token.as_rule() {
                    Rule::assignment_exp => {
                        println!("Assignment: {}", token.as_str());
                        Self::parse_assignment(token)?;
                    }
                    // Rule::pipeline_statement => {
                    //     println!("Command: {}", token.as_str());
                    // }
                    _ => {
                        println!("not implemented: {:?}", token.as_rule());
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_statement(token: Pair<'a>) -> PestResult<()> {
        match token.as_rule() {
            Rule::assignment_exp => {
                println!("Assignment: {}", token.as_str());
                Self::parse_assignment(token)?;
            }
            // Rule::pipeline_statement => {
            //     println!("Command: {}", token.as_str());
            // }
            _ => {
                println!("not implemented: {:?}", token.as_rule());
            }
        }
        Ok(())
    }

    fn parse_assignment(token: Pair<'a>) -> PestResult<()> {
        let mut tokens = token.into_inner();
        let token_variable = tokens.next().expect("Failed to get token");
        if token_variable.as_rule() != Rule::variable {
            //todo
        }
        let token_expandable = tokens.next().expect("Failed to get token");
        match token_expandable.as_rule() {
            Rule::expandable_string_content => {
                println!("Expandable: {}", token_expandable.as_str());
                //Self::expand_string(token_expandable.as_str())?;
            }
            _ => {
                println!("not implemented: {:?}", token_expandable.as_rule());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PowerShellParser;

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

        let _ = PowerShellParser::parse_input(input).unwrap();
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

        let _ = PowerShellParser::parse_input(input).unwrap();
    }

    #[test]
    fn foreach_loop() {
        let input = r#"
foreach ($n in $numbers) {
    Write-Output $n
}
"#;

        let _ = PowerShellParser::parse_input(input).unwrap();
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

        let _ = PowerShellParser::parse_input(input).unwrap();
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

        let _ = PowerShellParser::parse_input(input).unwrap();
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

        let _ = PowerShellParser::parse_input(input).unwrap();
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

        let _ = PowerShellParser::parse_input(input).unwrap();
    }

    #[test]
    fn command() {
        let input = r#"
Get-Process | Where-Object { $_.CPU -gt 100 }
"#;

        let _ = PowerShellParser::parse_input(input).unwrap();
    }

    #[test]
    fn range() {
        let input = r#"
$numbers = 1..5
"#;

        let _ = PowerShellParser::parse_input(input).unwrap();
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

        let _ = PowerShellParser::parse_input(input).unwrap();
    }

    //#[test]
    fn floats() {
        let input = r#"
    $pi = 3.1415
$half = .5
"#;

        let _ = PowerShellParser::parse_input(input).unwrap();
    }

    #[test]
    fn arrays() {
        let input = r#"
$a = 1, 2, 3
$b = @("one", "two", "three")
$c = @(1, 2, @(3, 4))
"#;

        let _ = PowerShellParser::parse_input(input).unwrap();
    }

    #[test]
    fn amsi_fail() {
        let input = r#"
#Matt Graebers second Reflection method 
$VMRviwsbtehQfPtxbt=$null;
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Âmsí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))"

"#;

        let _ = PowerShellParser::parse_input(input).unwrap();
    }
}
