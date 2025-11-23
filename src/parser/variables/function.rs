use std::collections::HashMap;

use super::Variables;
use crate::parser::{
    ScriptBlock,
    command::{CallablePredType, CommandOutput},
};

pub(crate) type FunctionMap = HashMap<String, ScriptBlock>;
use crate::parser::CommandElem;
impl Variables {
    pub(crate) fn get_function(
        &mut self,
        name: &str,
    ) -> Option<CallablePredType<CommandElem, CommandOutput>> {
        if let Some(fun) = self.script_functions.get(name).cloned() {
            Self::get_function_from_script_block(fun)
        } else if let Some(fun) = self.global_functions.get(name).cloned() {
            Self::get_function_from_script_block(fun)
        } else {
            None
        }
    }

    pub(super) fn get_function_from_script_block(
        sb: ScriptBlock,
    ) -> Option<CallablePredType<CommandElem, CommandOutput>> {
        let fun = move |params, ps: &mut crate::PowerShellSession| {
            let sb = sb.clone();
            sb.run(params, ps, None)
        };
        Some(Box::new(fun))
    }
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn int() {
        let input = r#"
function Add-Numbers($a, $b) {
    return $a + $b
}
Add-Numbers 5 10
"#;

        let deobfuscated = r#"
function Add-Numbers($a, $b) {
    return $a + $b
}
15
"#;

        let mut session = PowerShellSession::new();
        let script_result = session.parse_input(input).unwrap();
        assert_eq!(script_result.result(), PsValue::Int(15));
        assert_eq!(script_result.deobfuscated(), deobfuscated.trim());
    }

    #[test]
    fn string() {
        let input = r#"
function Get-Greeting($name = "World") {
    return "Hello, $name!"
}
Get-Greeting
Get-Greeting "Alice"
        "#;

        let deobfuscated = r#"
function Get-Greeting($name = "World") {
    return "Hello, $name!"
}
"Hello, World!"
"Hello, Alice!"
"#;

        let mut session = PowerShellSession::new();
        let script_result = session.parse_input(input).unwrap();
        assert_eq!(script_result.deobfuscated().trim(), deobfuscated.trim());
        assert_eq!(
            script_result.result(),
            PsValue::String("Hello, Alice!".into())
        );
    }

    #[test]
    fn empty() {
        let input = r#"
function Test-Empty() {
}
Test-Empty
        "#;

        let deobfuscated = r#"
function Test-Empty() {
}
        "#;

        let mut session = PowerShellSession::new();
        let script_result = session.parse_input(input).unwrap();
        assert_eq!(script_result.deobfuscated().trim(), deobfuscated.trim());
        assert_eq!(script_result.result(), PsValue::Null);
    }

    #[test]
    fn params_block() {
        let input = r#"
function Test-Parameters {
    param(
        [string]$Name,
        [int]$Age = 25,
        [switch]$Verbose
    )
    
    $result = "Name: $Name, Age: $Age"
    if ($Verbose) {
        $result += " (Verbose mode)"
    }
    return $result
}
Test-Parameters -Name "Bob" -Age 30 -Verbose
"#;

        let deobfuscated = r#"
function Test-Parameters {
    param(
        [string]$Name,
        [int]$Age = 25,
        [switch]$Verbose
    )
    
    $result = "Name: $Name, Age: $Age"
    if ($Verbose) {
        $result += " (Verbose mode)"
    }
    return $result
}
"Name: Bob, Age: 30 (Verbose mode)"
"#;

        let mut session = PowerShellSession::new();
        let script_result = session.parse_input(input).unwrap();
        assert_eq!(script_result.deobfuscated().trim(), deobfuscated.trim());
    }

    #[test]
    fn global() {
        let input = r#"
        function global:Add-Numbers($a, $b) {
            return $a + $b
        }
        "#;

        let mut session = PowerShellSession::new();
        let _ = session.parse_input(input).unwrap();
        let script_result = session.parse_input("Add-Numbers 5 10").unwrap();
        assert_eq!(script_result.result(), PsValue::Int(15));
    }

    // #[test]
    // fn filter() {
    //     let input = r#"
    //     filter global:Get-Numbers ([switch]$EvenOnly) {
    //         if ($EvenOnly) { if ($_ % 2 -eq 0) { $_ } }
    //         else { $_ }
    //     }
    //     "#;

    //     let mut session = PowerShellSession::new();
    //     let _ = session.parse_input(input).unwrap();
    //     let script_result = session.parse_input("0..10 |
    // Get-Numbers").unwrap();     assert_eq!(script_result.deobfuscated(),
    // "0 1 2 3 4 5 6 7 8 9 10");

    //     let script_result = session
    //         .parse_input("0..10 | Get-Numbers -EvenOnly")
    //         .unwrap();
    //     assert_eq!(script_result.deobfuscated(), "0 2 4 6 8 10");
    // }
}
