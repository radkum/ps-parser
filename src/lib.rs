//! # ps-parser
//!
//! A fast and flexible PowerShell parser written in Rust.
//!
//! ## Overview
//!
//! `ps-parser` provides parsing, evaluation, and manipulation of PowerShell
//! scripts. It supports variables, arrays, hash tables, script blocks,
//! arithmetic, logical operations, and more.
//!
//! ## Features
//!
//! - Parse PowerShell scripts using pest grammar
//! - Evaluate expressions, variables, arrays, hash tables, and script blocks
//! - Environment and INI variable loading
//! - Deobfuscation and error reporting
//! - Extensible for custom PowerShell types
//!
//! ## Usage
//!
//! ```rust
//! use ps_parser::PowerShellSession;
//!
//! let mut session = PowerShellSession::new();
//! let output = session.safe_eval(r#"$a = 42; Write-Output $a"#).unwrap();
//! println!("{}", output); // prints: 42
//! ```

mod parser;
pub(crate) use parser::NEWLINE;
/// Represents a PowerShell parsing and evaluation session.
///
/// This is the main entry point for parsing and evaluating PowerShell scripts.
/// It maintains the session state including variables, tokens, and error
/// information.
///
/// # Examples
///
/// ```rust
/// use ps_parser::PowerShellSession;
///
/// // Create a new session
/// let mut session = PowerShellSession::new();
///
/// // Evaluate a simple expression
/// let result = session.safe_eval("$a = 1 + 2; Write-Output $a").unwrap();
/// assert_eq!(result, "3");
///
/// // Parse and get detailed results
/// let script_result = session.parse_input("$b = 'Hello World'; $b").unwrap();
/// println!("Result: {:?}", script_result.result());
/// ```
pub use parser::PowerShellSession;
/// Represents a PowerShell value that can be stored and manipulated.
///
/// This enum covers all the basic PowerShell data types including primitives,
/// collections, and complex objects like script blocks and hash tables.
///
/// # Examples
///
/// ```rust
/// use ps_parser::PsValue;
///
/// // Different value types  
/// let int_val = PsValue::Int(42);
/// let string_val = PsValue::String("Hello".into());
/// let bool_val = PsValue::Bool(true);
/// ```
pub use parser::PsValue;
/// Contains the complete result of parsing and evaluating a PowerShell script.
///
/// This structure holds the final result value, any output generated,
/// parsing errors encountered, and the tokenized representation of the script.
/// It's particularly useful for debugging and deobfuscation purposes.
///
/// # Examples
///
/// ```rust
/// use ps_parser::PowerShellSession;
///
/// let mut session = PowerShellSession::new();
/// let script_result = session.parse_input("$a = 42; $a").unwrap();
///
/// // Access different parts of the result
/// println!("Final value: {:?}", script_result.result());
/// println!("Output: {:?}", script_result.output());
/// println!("Errors: {:?}", script_result.errors());
/// ```
pub use parser::ScriptResult;
/// Represents a parsed token from a PowerShell script.
///
/// Tokens are the building blocks of parsed PowerShell code and are used
/// for syntax analysis, deobfuscation, and code transformation.
///
/// Right now 4 token types are supported:
/// - **String**: Representation of single quoted PowerShell strings (e.g.,
///   `'hello world'`)
/// - **StringExpandable**: Representation of double quoted PowerShell strings
///   with variable expansion (e.g., `"Hello $name"`)
/// - **Expression**: Parsed PowerShell expressions with their evaluated results
///   (e.g., `$a + $b`)
/// - **Function**: PowerShell function definitions and calls
///
/// Each token type stores both the original source code and its
/// processed/evaluated form, making it useful for deobfuscation and analysis
/// purposes.
///
/// # Examples
///
/// ```rust
/// use ps_parser::PowerShellSession;
///
/// let mut session = PowerShellSession::new();
/// let script_result = session.parse_input("$var = 123").unwrap();
///
/// // Inspect the tokens
/// for token in script_result.tokens().all() {
///     println!("Token: {:?}", token);
/// }
/// ```
pub use parser::Token;
/// Manages PowerShell variables across different scopes.
///
/// This structure handles variable storage, retrieval, and scope management
/// for PowerShell scripts. It supports loading variables from environment
/// variables, INI files, and manual assignment.
///
/// # Examples
///
/// ```rust
/// use ps_parser::{Variables, PowerShellSession};
/// use std::path::Path;
///
/// // Load environment variables
/// let env_vars = Variables::env();
/// let mut session = PowerShellSession::new().with_variables(env_vars);
///
/// // Load from INI string
/// let ini_vars = Variables::from_ini_string("[global]\nname = John Doe\n[local]\nlocal_var = \"local_value\"").unwrap();
/// let mut session2 = PowerShellSession::new().with_variables(ini_vars);
///
/// // Create empty and add manually
/// let mut vars = Variables::new();
/// // ... add variables manually
/// ```
pub use parser::Variables;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::Token;

    #[test]
    fn obfuscation_1() {
        let input = r#"
$ilryNQSTt="System.$([cHAR]([ByTE]0x4d)+[ChAR]([byte]0x61)+[chAr](110)+[cHar]([byTE]0x61)+[cHaR](103)+[cHar](101*64/64)+[chaR]([byTE]0x6d)+[cHAr](101)+[CHAr]([byTE]0x6e)+[Char](116*103/103)).$([Char]([ByTe]0x41)+[Char](117+70-70)+[CHAr]([ByTE]0x74)+[CHar]([bYte]0x6f)+[CHar]([bytE]0x6d)+[ChaR]([ByTe]0x61)+[CHar]([bYte]0x74)+[CHAR]([byte]0x69)+[Char](111*26/26)+[chAr]([BYTe]0x6e)).$(('Ârmí'+'Ùtìl'+'s').NORmalizE([ChAR](44+26)+[chAR](111*9/9)+[cHar](82+32)+[ChaR](109*34/34)+[cHaR](68+24-24)) -replace [ChAr](92)+[CHaR]([BYTe]0x70)+[Char]([BytE]0x7b)+[CHaR]([BYTe]0x4d)+[chAR](110)+[ChAr](15+110))";$ilryNQSTt
"#;

        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(input).unwrap().as_str(),
            "System.Management.Automation.ArmiUtils"
        );
    }

    #[test]
    fn obfuscation_2() {
        let input = r#"
$(('W'+'r'+'î'+'t'+'é'+'Í'+'n'+'t'+'3'+'2').NormAlIzE([chaR]([bYTE]0x46)+[CHAR](111)+[ChAR]([Byte]0x72)+[CHAR]([BytE]0x6d)+[CHAr](64+4)) -replace [cHAr]([BytE]0x5c)+[char]([bYtE]0x70)+[ChAR]([byTe]0x7b)+[cHar]([bYtE]0x4d)+[Char]([bYte]0x6e)+[CHAR](125))
"#;

        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(input).unwrap().as_str(), "WriteInt32");
    }

    #[test]
    fn obfuscation_3() {
        let input = r#"
$([cHar]([BYte]0x65)+[chAr]([bYTE]0x6d)+[CHaR]([ByTe]0x73)+[char](105)+[CHAR]([bYTE]0x43)+[cHaR](111)+[chaR]([bYTE]0x6e)+[cHAr]([bYTe]0x74)+[cHAr](32+69)+[cHaR](120+30-30)+[cHAR]([bYte]0x74))
"#;

        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(input).unwrap().as_str(), "emsiContext");
    }

    #[test]
    fn obfuscation_4() {
        let input = r#"
[syStem.texT.EncoDInG]::unIcoDe.geTstRiNg([SYSTem.cOnVERT]::froMbasE64striNg("WwBjAGgAYQByAF0AKABbAGkAbgB0AF0AKAAiADkAZQA0AGUAIgAgAC0AcgBlAHAAbABhAGMAZQAgACIAZQAiACkAKwAzACkA"))"#;

        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(input).unwrap().as_str(),
            r#"[char]([int]("9e4e" -replace "e")+3)"#
        );
    }

    #[test]
    fn deobfuscation() {
        // assign variable and print it to screen
        let mut p = PowerShellSession::new();
        let input = r#" $global:var = [char]([int]("9e4e" -replace "e")+3); [int]'a';$var"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), 'a'.into());
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$var = 'a'", "[int]'a'"].join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 1);
        assert_eq!(
            script_res.errors()[0].to_string(),
            "ValError: Cannot convert value \"String\" to type \"Int\""
        );

        // the same but do it in two parts
        let mut p = PowerShellSession::new();
        let input = r#" $global:var = [char]([int]("9e4e" -replace "e")+3) "#;
        let script_res = p.parse_input(input).unwrap();

        assert_eq!(script_res.errors().len(), 0);

        let script_res = p.parse_input(" [int]'a';$var ").unwrap();
        assert_eq!(script_res.deobfuscated(), vec!["[int]'a'"].join(NEWLINE));
        assert_eq!(script_res.output(), vec!["a"].join(NEWLINE));
        assert_eq!(script_res.errors().len(), 1);
        assert_eq!(
            script_res.errors()[0].to_string(),
            "ValError: Cannot convert value \"String\" to type \"Int\""
        );
    }

    #[test]
    fn deobfuscation_non_existing_value() {
        // assign not existing value, without forcing evaluation
        let mut p = PowerShellSession::new();
        let input = r#" $local:var = $env:programfiles;[int]'a';$var"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$local:var = $env:programfiles", "[int]'a'", "$var"].join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 3);
        assert_eq!(
            script_res.errors()[0].to_string(),
            "VariableError: Variable \"programfiles\" is not defined"
        );
        assert_eq!(
            script_res.errors()[1].to_string(),
            "ValError: Cannot convert value \"String\" to type \"Int\""
        );
        assert_eq!(
            script_res.errors()[2].to_string(),
            "VariableError: Variable \"var\" is not defined"
        );

        // assign not existing value, forcing evaluation
        let mut p = PowerShellSession::new().with_variables(Variables::force_eval());
        let input = r#" $global:var = $env:programfiles;[int]'a';$var"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$var = $null", "[int]'a'"].join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 1);
    }

    #[test]
    fn deobfuscation_env_value() {
        // assign not existing value, without forcing evaluation
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" $global:var = $env:programfiles;$var"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String(std::env::var("PROGRAMFILES").unwrap())
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec![format!(
                "$var = '{}'",
                std::env::var("PROGRAMFILES").unwrap()
            )]
            .join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn hash_table() {
        // assign not existing value, without forcing evaluation
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" 
$nestedData = @{
    Users = @(
        @{ Name = "Alice"; Age = 30; Skills = @("PowerShell", "Python") }
        @{ Name = "Bob"; Age = 25; Skills = @("Java", "C#") }
    )
    Settings = @{
        Theme = "Dark"
        Language = "en-US"
    }
}
"$nestedData"
        "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("System.Collections.Hashtable".to_string())
        );

        assert_eq!(
            p.parse_input("$nesteddata.settings").unwrap().result(),
            PsValue::HashTable(HashMap::from([
                ("language".to_string(), PsValue::String("en-US".to_string())),
                ("theme".to_string(), PsValue::String("Dark".to_string())),
            ]))
        );

        assert_eq!(
            p.safe_eval("$nesteddata.settings.theme").unwrap(),
            "Dark".to_string()
        );

        assert_eq!(
            p.parse_input("$nesteddata.users[0]").unwrap().result(),
            PsValue::HashTable(HashMap::from([
                (
                    "skills".to_string(),
                    PsValue::Array(vec![
                        PsValue::String("PowerShell".to_string()),
                        PsValue::String("Python".to_string().into())
                    ])
                ),
                ("name".to_string(), PsValue::String("Alice".to_string())),
                ("age".to_string(), PsValue::Int(30)),
            ]))
        );

        assert_eq!(
            p.safe_eval("$nesteddata.users[0]['name']").unwrap(),
            "Alice".to_string()
        );

        assert_eq!(
            p.safe_eval("$nesteddata.users[0].NAME").unwrap(),
            "Alice".to_string()
        );
    }

    #[test]
    fn test_simple_arithmetic() {
        let input = r#"
Write-Host "=== Test 3: Arithmetic Operations ===" -ForegroundColor Green
$a = 10
$b = 5
Write-Output "Addition: $(($a + $b))"
Write-Output "Subtraction: $(($a - $b))"
Write-Output "Multiplication: $(($a * $b))"
Write-Output "Division: $(($a / $b))"
Write-Output "Modulo: $(($a % $b))"
"#;

        let script_result = PowerShellSession::new().parse_input(input).unwrap();

        assert_eq!(script_result.result(), PsValue::String("Modulo: 0".into()));
        assert_eq!(
            script_result.output(),
            vec![
                r#"=== Test 3: Arithmetic Operations ==="#,
                r#"Addition: 15"#,
                r#"Subtraction: 5"#,
                r#"Multiplication: 50"#,
                r#"Division: 2"#,
                r#"Modulo: 0"#
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_result.errors().len(), 0);
        assert_eq!(script_result.tokens().strings(), vec![]);
        assert_eq!(script_result.tokens().expandable_strings().len(), 6);
        assert_eq!(
            script_result.tokens().expandable_strings()[1],
            Token::StringExpandable(
                "\"Addition: $(($a + $b))\"".to_string(),
                "Addition: 15".to_string()
            )
        );
        assert_eq!(script_result.tokens().expression().len(), 12);
        assert_eq!(
            script_result.tokens().expression()[2],
            Token::Expression("$a + $b".to_string(), PsValue::Int(15))
        );
    }

    #[test]
    fn test_scripts() {
        use std::fs;
        let Ok(entries) = fs::read_dir("test_scripts") else {
            panic!("Failed to read test files");
        };
        for entry in entries {
            let dir_entry = entry.unwrap();
            if std::fs::FileType::is_dir(&dir_entry.file_type().unwrap()) {
                // If it's a directory, we can read the files inside it
                let input_script = dir_entry.path().join("input.ps1");
                let deobfuscated = dir_entry.path().join("deobfuscated.txt");
                let output = dir_entry.path().join("output.txt");

                let Ok(content) = fs::read_to_string(&input_script) else {
                    panic!("Failed to read test files");
                };

                let Ok(deobfuscated) = fs::read_to_string(&deobfuscated) else {
                    panic!("Failed to read test files");
                };

                let Ok(output) = fs::read_to_string(&output) else {
                    panic!("Failed to read test files");
                };

                let script_result = PowerShellSession::new()
                    .with_variables(Variables::env())
                    .parse_input(&content)
                    .unwrap();

                let deobfuscated_vec = deobfuscated
                    .lines()
                    .map(|s| s.trim_end())
                    .collect::<Vec<&str>>();

                let script_deobfuscated = script_result.deobfuscated();

                let output_vec = output.lines().map(|s| s.trim_end()).collect::<Vec<&str>>();

                let script_output = script_result.output();

                let _name = dir_entry
                    .path()
                    .components()
                    .last()
                    .unwrap()
                    .as_os_str()
                    .to_string_lossy()
                    .to_string();
                // std::fs::write(
                //     format!("{}_deobfuscated.txt", _name),
                //     script_deobfuscated.clone(),
                // )
                // .unwrap();
                // std::fs::write(format!("{}_output.txt", _name),
                // script_output.clone()).unwrap();
                let script_deobfuscated_vec = script_deobfuscated
                    .lines()
                    .map(|s| s.trim_end())
                    .collect::<Vec<&str>>();

                let script_output_vec = script_output
                    .lines()
                    .map(|s| s.trim_end())
                    .collect::<Vec<&str>>();

                for i in 0..deobfuscated_vec.len() {
                    assert_eq!(deobfuscated_vec[i], script_deobfuscated_vec[i]);
                }

                for i in 0..output_vec.len() {
                    assert_eq!(output_vec[i], script_output_vec[i]);
                }
            }
        }
    }

    #[test]
    fn test_range() {
        // Test for even numbers
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" $numbers = 1..10; $numbers"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$numbers = @(1,2,3,4,5,6,7,8,9,10)"].join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn even_numbers() {
        // Test for even numbers
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" $numbers = 1..10; $evenNumbers = $numbers | Where-Object { $_ % 2 -eq 0 }; $evenNumbers"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::Int(2),
                PsValue::Int(4),
                PsValue::Int(6),
                PsValue::Int(8),
                PsValue::Int(10)
            ])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                "$numbers = @(1,2,3,4,5,6,7,8,9,10)",
                "$evennumbers = @(2,4,6,8,10)"
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }

    //#[test]
    fn _test_function() {
        // Test for even numbers
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" 
function Get-Square($number) {
    return $number * $number
}
"Square of 5: $(Get-Square 5)" "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                "function Get-Square($number) {",
                "    return $number * $number",
                "}",
                " \"Square of 5: $(Get-Square 5)\""
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 2);
    }

    #[test]
    fn test_if() {
        // Test for even numbers
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" 
        # Test 10: Conditional Statements
if ($true) {
    $if_result = "condition true"
}

if ($false) {
    $else_result = "false branch"
} else {
    $else_result = "true branch"
}

$score = 85
if ($score -ge 90) {
    $grade = "A"
} elseif ($score -ge 80) {
    $grade = "B"
} else {
    $grade = "C"
}
        
        "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                "$if_result = 'condition true'",
                "$else_result = 'true branch'",
                "$score = 85",
                "$grade = 'B'"
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }
}
