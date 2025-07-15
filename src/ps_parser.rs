use pest::Parser;
use pest_derive::Parser;

type PestError = pest::error::Error<Rule>;
type PestResult<T> = core::result::Result<T, PestError>;
type Pair<'i> = ::pest::iterators::Pair<'i, Rule>;

#[derive(Parser)]
#[grammar = "powershell.pest"]
pub(crate) struct PowerShellParser;

impl PowerShellParser {
    pub fn parse_input(input: &str) -> PestResult<()> {
        let mut pairs = PowerShellParser::parse(Rule::program, input)?;
        let program_token = pairs.next().expect("");

        if let Rule::program = program_token.as_rule() {
            let pairs = program_token.into_inner();
            for pair in pairs {
                Self::parse_statement(pair)?;
            }
        }

        Ok(())
    }

    fn parse_statement<'a>(token: Pair<'a>) -> PestResult<()> {
        match token.as_rule() {
            Rule::assignment => {
                println!("Assignment: {}", token.as_str());
                Self::parse_assignment(token)?;
            }
            Rule::pipeline => {
                println!("Command: {}", token.as_str());
            }
            _ => {
                println!("not implemented: {:?}", token.as_rule());
            }
        }
        Ok(())
    }

    fn parse_assignment<'a>(token: Pair<'a>) -> PestResult<()> {
        let mut tokens = token.into_inner();
        let token_variable = tokens.next().expect("Failed to get token");
        if token_variable.as_rule() != Rule::variable {
            //todo
        }
        let token_expandable = tokens.next().expect("Failed to get token");
        match token_expandable.as_rule() {
            Rule::expandable_string_content => {
                println!("Expandable: {}", token_expandable.as_str());
                //expand_string(token_expandable.as_str())?;
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
    $pi = 3.1415
$half = .5
$hex = 0xFF
$bin = 0b1101

$name = "Alice"
$msg = "Hello, $name. Today is $day."
$escaped = "She said: `"Hi`""
$literal = 'Hello, $name'
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
