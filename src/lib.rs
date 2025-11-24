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
pub use parser::{CommandToken, ExpressionToken, MethodToken, StringExpandableToken};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{ExpressionToken, StringExpandableToken};

    #[test]
    fn deobfuscation() {
        // assign variable and print it to screen
        let mut p = PowerShellSession::new();
        let input = r#" $script:var = [char]([int]("9e4e" -replace "e")+3); [int]'a';$var"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), 'a'.into());
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$script:var = 'a'", "[int]'a'", "'a'"].join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 1);
        assert_eq!(
            script_res.errors()[0].to_string(),
            "ValError: Failed to convert value \"a\" to type Int"
        );

        // the same but do it in two parts
        let mut p = PowerShellSession::new();
        let input = r#" $global:var = [char]([int]("9e4e" -replace "e")+3) "#;
        let script_res = p.parse_input(input).unwrap();

        assert_eq!(script_res.errors().len(), 0);

        let script_res = p.parse_input(" [int]'a';$var ").unwrap();
        assert_eq!(
            script_res.deobfuscated(),
            vec!["[int]'a'", "'a'"].join(NEWLINE)
        );
        assert_eq!(script_res.output(), vec!["a"].join(NEWLINE));
        assert_eq!(script_res.errors().len(), 1);
        assert_eq!(
            script_res.errors()[0].to_string(),
            "ValError: Failed to convert value \"a\" to type Int"
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
            "ValError: Failed to convert value \"a\" to type Int"
        );
        assert_eq!(
            script_res.errors()[2].to_string(),
            "VariableError: Variable \"var\" is not defined"
        );

        // assign not existing value, forcing evaluation
        let mut p = PowerShellSession::new().with_variables(Variables::force_eval());
        let input = r#" $local:var = $env:programfiles;[int]'a';$script:var"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$local:var = $null", "[int]'a'"].join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 1);
    }

    #[test]
    fn deobfuscation_env_value() {
        // assign not existing value, without forcing evaluation
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" $local:var = $env:programfiles;$var"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String(std::env::var("PROGRAMFILES").unwrap())
        );
        let program_files = std::env::var("PROGRAMFILES").unwrap();
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                format!("$local:var = \"{}\"", program_files),
                format!("\"{}\"", program_files)
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn deobfuscation_from_base_64() {
        let mut p = PowerShellSession::new();
        let input = r#" $encoded = [syStem.texT.EncoDInG]::unIcoDe.geTstRiNg([char]97);$encoded"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), String::from("\u{FFFD}").into());

        let input = r#" [syStem.texT.EncoDInG]::unIcoDe.geTstRiNg([SYSTem.cOnVERT]::froMbasE64striNg("ZABlAGMAbwBkAGUAZAA="))"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), String::from("decoded").into());
    }

    #[test]
    fn hash_table() {
        // assign not existing value, without forcing evaluation
        let mut p = PowerShellSession::new().with_variables(Variables::env().values_persist());
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

        let input = r#" $a=@{val = 4};$a.val"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(4));
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$a = @{", "\tval = 4", "}", "4"].join(NEWLINE)
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
        assert_eq!(script_result.tokens().expandable_strings().len(), 6);
        assert_eq!(
            script_result.tokens().expandable_strings()[1],
            StringExpandableToken::new(
                "\"Addition: $(($a + $b))\"".to_string(),
                "Addition: 15".to_string()
            )
        );
        assert_eq!(script_result.tokens().expressions().len(), 12);
        assert_eq!(
            script_result.tokens().expressions()[2],
            ExpressionToken::new("$a + $b".to_string(), PsValue::Int(15))
        );
    }

    #[test]
    fn test_scripts() {
        use std::fs;
        let Ok(entries) = fs::read_dir("test_scripts") else {
            panic!("Failed to read 'test_scripts' directory");
        };
        for entry in entries {
            let dir_entry = entry.unwrap();
            if std::fs::FileType::is_dir(&dir_entry.file_type().unwrap()) {
                // If it's a directory, we can read the files inside it
                let input_script = dir_entry.path().join("input.ps1");
                let expected_deobfuscated_script = dir_entry.path().join("deobfuscated.txt");
                let expected_output_script = dir_entry.path().join("output.txt");

                let Ok(input) = fs::read_to_string(&input_script) else {
                    panic!("Failed to read test file: {}", input_script.display());
                };

                let Ok(expected_deobfuscated) = fs::read_to_string(&expected_deobfuscated_script)
                else {
                    panic!(
                        "Failed to read test file: {}",
                        expected_deobfuscated_script.display()
                    );
                };

                let Ok(expected_output) = fs::read_to_string(&expected_output_script) else {
                    panic!(
                        "Failed to read test file: {}",
                        expected_output_script.display()
                    );
                };

                let script_result = PowerShellSession::new()
                    .with_variables(Variables::env())
                    .parse_input(&input)
                    .unwrap();

                let expected_deobfuscated_vec = expected_deobfuscated
                    .lines()
                    .map(|s| s.trim_end())
                    .collect::<Vec<&str>>();

                let current_deobfuscated = script_result.deobfuscated();
                let current_output = script_result.output();

                let expected_output_vec = expected_output
                    .lines()
                    .map(|s| s.trim_end())
                    .collect::<Vec<&str>>();

                //save_files(&dir_entry, &current_deobfuscated, &current_output);
                let current_deobfuscated_vec = current_deobfuscated
                    .lines()
                    .map(|s| s.trim_end())
                    .collect::<Vec<&str>>();

                let current_output_vec = current_output
                    .lines()
                    .map(|s| s.trim_end())
                    .collect::<Vec<&str>>();

                for i in 0..expected_deobfuscated_vec.len() {
                    assert_eq!(
                        expected_deobfuscated_vec[i],
                        current_deobfuscated_vec[i],
                        "File: {}, Deobfuscated line: {}",
                        file_name(&dir_entry),
                        i + 1
                    );
                }

                for i in 0..expected_output_vec.len() {
                    assert_eq!(
                        expected_output_vec[i],
                        current_output_vec[i],
                        "File: {}, Output line: {}",
                        file_name(&dir_entry),
                        i + 1
                    );
                }
            }
        }
    }

    fn file_name(dir_entry: &std::fs::DirEntry) -> String {
        dir_entry
            .path()
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_string_lossy()
            .to_string()
    }

    #[allow(dead_code)]
    fn save_files(dir_entry: &std::fs::DirEntry, deobfuscated: &str, output: &str) {
        let name = file_name(dir_entry);
        std::fs::write(format!("{}_deobfuscated.txt", name), deobfuscated).unwrap();
        std::fs::write(format!("{}_output.txt", name), output).unwrap();
    }

    #[test]
    fn test_range() {
        // Test for even numbers
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" $numbers = 1..10; $numbers"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                "$numbers = @(1,2,3,4,5,6,7,8,9,10)",
                "@(1,2,3,4,5,6,7,8,9,10)"
            ]
            .join(NEWLINE)
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
                "$evennumbers = @(2,4,6,8,10)",
                "@(2,4,6,8,10)"
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn divisible_by_2_and_3() {
        // Test for even numbers
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" $numbers = 1..10; $numbers | Where { $_ % 2 -eq 0 } | ? { $_ % 3 -eq 0 }"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(6));
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$numbers = @(1,2,3,4,5,6,7,8,9,10)", "6"].join(NEWLINE)
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
                "$if_result = \"condition true\"",
                "$else_result = \"true branch\"",
                "$score = 85",
                "$grade = \"B\""
            ]
            .join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn format_operator() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" ("{5}{2}{0}{1}{3}{6}{4}" -f 'ut',('oma'+'t'+'ion.'),'.A',('Ems'+'iUt'),'ls',('S'+'ystem.'+'Danage'+'men'+'t'),'i')"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("System.Danagement.Automation.EmsiUtils".into())
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec![r#""System.Danagement.Automation.EmsiUtils""#].join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn encod_command() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());
        let input = r#" ("{5}{2}{0}{1}{3}{6}{4}" -f 'ut',('oma'+'t'+'ion.'),'.A',('Ems'+'iUt'),'ls',('S'+'ystem.'+'Danage'+'men'+'t'),'i')"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("System.Danagement.Automation.EmsiUtils".into())
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec![r#""System.Danagement.Automation.EmsiUtils""#].join(NEWLINE)
        );
        assert_eq!(script_res.errors().len(), 0);
    }

    #[test]
    fn array_literals() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());

        //integers
        let input = r#" $a = 1,2,3;$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Int(1), PsValue::Int(2), PsValue::Int(3)])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$a = @(1,2,3)", "@(1,2,3)"].join(NEWLINE)
        );

        // strings
        let input = r#" $a = "x", 'yyy', "z";$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::String("x".into()),
                PsValue::String("yyy".into()),
                PsValue::String("z".into())
            ])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec![r#"$a = @("x","yyy","z")"#, r#"@("x","yyy","z")"#].join(NEWLINE)
        );

        // expresssions
        let input = r#" $a = 1,2+ 3,[long]4;$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::Int(1),
                PsValue::Int(2),
                PsValue::Int(3),
                PsValue::Int(4),
            ])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$a = @(1,2,3,4)", "@(1,2,3,4)"].join(NEWLINE)
        );

        // variables
        let input = r#" $x = 3; $a = $x, $x+1, "count=$x";$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::Int(3),
                PsValue::Int(3),
                PsValue::Int(1),
                PsValue::String("count=3".into()),
            ])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                "$x = 3",
                "$a = @(3,3,1,\"count=3\")",
                "@(3,3,1,\"count=3\")"
            ]
            .join(NEWLINE)
        );

        // nested arrays
        let input = r#" $a = (1, 2), (3, 4);$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::Array(vec![PsValue::Int(1), PsValue::Int(2)]),
                PsValue::Array(vec![PsValue::Int(3), PsValue::Int(4)]),
            ])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$a = @(@(1,2),@(3,4))", "@(@(1,2),@(3,4))"].join(NEWLINE)
        );

        // nested arrays
        let input = r#" $a = 1, "two", 3.0, $false, (Get-Date);$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::Int(1),
                PsValue::String("two".into()),
                PsValue::Float(3.0),
                PsValue::Bool(false),
                PsValue::String("Get-Date".into()),
            ])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec![
                "$a = @(1,\"two\",3,$false,Get-Date)",
                "@(1,\"two\",3,$false,Get-Date)"
            ]
            .join(NEWLINE)
        );

        // array assign to another array
        let input = r#" $a = 1, 2,3;$b = $a,4,5;$b"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::Array(vec![PsValue::Int(1), PsValue::Int(2), PsValue::Int(3)]),
                PsValue::Int(4),
                PsValue::Int(5),
            ])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$a = @(1,2,3)", "$b = @(@(1,2,3),4,5)", "@(@(1,2,3),4,5)"].join(NEWLINE)
        );

        // forEach-Object
        let input = r#"  $a = 1,-2,(-3) | ForEach-Object { $_ * 2 };$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Int(2), PsValue::Int(-4), PsValue::Int(-6),])
        );

        // forEach-Object - parentheses
        let input = r#"  $a = (1,2,3) | ForEach-Object { $_ };$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Int(1), PsValue::Int(2), PsValue::Int(3),])
        );
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$a = @(1,2,3)", "@(1,2,3)"].join(NEWLINE)
        );

        // array assign to another array
        let input = r#" $a = @{
    A = 1,2,3
    B = (4,5),6
}
$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::HashTable(HashMap::from([
                (
                    "a".into(),
                    PsValue::Array(vec![PsValue::Int(1), PsValue::Int(2), PsValue::Int(3),])
                ),
                (
                    "b".into(),
                    PsValue::Array(vec![
                        PsValue::Array(vec![PsValue::Int(4), PsValue::Int(5)]),
                        PsValue::Int(6),
                    ])
                ),
            ]))
        );

        // function argument as array
        let input = r#" function Foo($x) { $x.GetType().name + $x[2]};Foo(1,2,3)"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("Object[]3".into()));

        // function argument as array
        let input = r#" [object[]](1,2,3)"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Int(1), PsValue::Int(2), PsValue::Int(3)])
        );

        // function argument as array
        let input = r#" $a = ,(42,2);$a"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Array(vec![
                PsValue::Int(42),
                PsValue::Int(2)
            ])])
        );

        // function argument as array
        let input = r#" function Foo($x) { $x.GetType().name + $x[2]};Foo(1,2,3)"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("Object[]3".into()));

        // function argument as array
        let input = r#" function b($x) {$x};b(1,2+3,4)"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::Int(1),
                PsValue::Int(2),
                PsValue::Int(3),
                PsValue::Int(4),
            ])
        );

        // function argument as array
        let input =
            r#" $a=@{val = 4};function b($x) {$x};b(1, [long]($a | Where-Object val -eq 4).val)"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Int(1), PsValue::Int(4)])
        );
    }

    #[test]
    fn cast_expression() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());

        //simple
        let input = r#" $a=@{val = 4};[long]($a).val"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(4));
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$a = @{", "\tval = 4", "}", "4"].join(NEWLINE)
        );

        let input = r#" $a=@{val = 4};[long]($a | Where-Object Val -eq 4).val"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(4));
        assert_eq!(
            script_res.deobfuscated(),
            vec!["$a = @{", "\tval = 4", "}", "4"].join(NEWLINE)
        );
    }

    #[test]
    fn as_expression() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());

        //simple
        let input = r#" '1a1' -replace 'a' -as [int] "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(11));

        let input = r#" '1a1' -replace ('a' -as [int])"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("1a1".into()));

        let input = r#" '2' -as [int] -shl 1"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(4));

        let input = r#" [system.text.encoding]::unicode -shl 1 "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.errors()[0].to_string(),
            String::from("BitwiseError: -shl not defined for UnicodeEncoding")
        );

        let input = r#" [int] -shl 1 "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.errors()[0].to_string(),
            String::from("BitwiseError: -shl not defined for Int32")
        );

        let input = r#" '2' -as ([string] -shl 1) "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.errors()[0].to_string(),
            String::from("BitwiseError: -shl not defined for String")
        );

        let input = r#" '2' -as ([int]) "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(2));

        let input = r#" '2' -As ([int]) "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(2));
    }

    #[test]
    fn cast_assignment() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());

        let input = r#" [int] $elo = "1"; $elo "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Int(1));

        let input = r#" [int] $elo = "1a": $elo"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.errors()[0].to_string(),
            String::from("ValError: Failed to convert value \"1a\" to type Int")
        );

        let input = r#" [double] $elo = "1a": $elo"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.errors()[0].to_string(),
            String::from("ValError: Failed to convert value \"1a\" to type Float")
        );

        let input = r#" [int[]] $elo = "1", "2"; $elo"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Int(1), PsValue::Int(2)])
        );

        let input = r#" [byte[]] $elo = "1", "2"; $elo"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Char(49), PsValue::Char(50)])
        );
    }

    #[test]
    fn splatten_arg() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());

        let input = r#" $a = @{ elo= 2; name= "radek"}; write-output @a "#;
        let script_res = p.parse_input(input).unwrap();
        assert!(script_res.output().contains("-elo 2"));
        assert!(script_res.output().contains("-name radek"));
    }

    #[test]
    fn strange_assignment() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());

        let input = r#" @(1,2)[0] = 1 "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.errors()[0].to_string(), "Skip".to_string());

        let input = r#" "elo"[0] = 1 "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.errors()[0].to_string(), "Skip".to_string());

        let input = r#" $a = @(1,2); $a[1] = 5; $a "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Int(1), PsValue::Int(5)])
        );

        let input = r#" $a = @(1,@(2,3));$a[1] = 6;$a "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::Int(1), PsValue::Int(6)])
        );

        let input = r#" $a = @(1,@(2,3));$a[1][1] = 6;$a "#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::Int(1),
                PsValue::Array(vec![PsValue::Int(2), PsValue::Int(6)])
            ])
        );
    }

    #[test]
    fn script_param_block() {
        let mut p = PowerShellSession::new().with_variables(Variables::env());

        let input = r#" 
[CmdletBinding(DefaultParameterSetName = "Path", HelpURI = "https://go.microsoft.com/fwlink/?LinkId=517145")]
param(
	[Parameter(ParameterSetName="Path", Position = 0)]
	[System.String[]]
	$Path


)

begin
{
	# Construct the strongly-typed crypto object
}

process
{
	Write-output elo
}
"#;
        let _script_res = p.parse_input(input).unwrap();
    }
}
