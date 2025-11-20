use std::vec;

use super::{MethodError, MethodResult, PsString, Val, ValType};

impl PsString {
    fn trim_arg(mut arg: Val) -> Option<Vec<char>> {
        if arg.ttype() == ValType::String {
            arg = Val::Array(vec![arg]);
        }
        let Val::Array(box_vec) = arg else {
            return None;
        };

        let mut trim_chars: Vec<char> = Vec::new();
        for v in box_vec.iter() {
            match v {
                Val::Char(c) => trim_chars.push(*c as u8 as char),
                Val::String(s) => {
                    let PsString(str) = s;
                    for ch in str.chars() {
                        trim_chars.push(ch);
                    }
                }
                _ => {
                    return None;
                }
            }
        }
        Some(trim_chars)
    }

    pub(super) fn trim(&self, args: Vec<Val>) -> MethodResult<Val> {
        if args.len() != 0 && args.len() != 1 {
            return Err(MethodError::new_incorrect_args("trim", args));
        }
        let PsString(input) = self;

        if args.is_empty() {
            return Ok(Val::String(input.trim().into()));
        } else if args.len() == 1 {
            let Some(trim_chars) = Self::trim_arg(args[0].clone()) else {
                return Err(MethodError::new_incorrect_args("trim", args));
            };

            return Ok(Val::String(input.trim_matches(&trim_chars[..]).into()));
        }
        Ok(Val::String(input.trim().into()))
    }

    pub(super) fn trim_start(&self, args: Vec<Val>) -> MethodResult<Val> {
        if args.len() != 1 {
            return Err(MethodError::new_incorrect_args("trim_start", args));
        }
        let PsString(input) = self;

        let Some(trim_chars) = Self::trim_arg(args[0].clone()) else {
            return Err(MethodError::new_incorrect_args("trim_start", args));
        };

        Ok(Val::String(
            input.trim_start_matches(&trim_chars[..]).into(),
        ))
    }

    pub(super) fn trim_end(&self, args: Vec<Val>) -> MethodResult<Val> {
        if args.len() != 1 {
            return Err(MethodError::new_incorrect_args("trim_end", args));
        }
        let PsString(input) = self;

        let Some(trim_chars) = Self::trim_arg(args[0].clone()) else {
            return Err(MethodError::new_incorrect_args("trim_end", args));
        };

        Ok(Val::String(input.trim_end_matches(&trim_chars[..]).into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn trim() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello, world'
$string = $string.trim(', world')
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("he".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.trim('hed')
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("llo, worl".to_string())
        );

        let input = r#"
$string = 'hello, world '
$string = $string.trim()
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("hello, world".to_string())
        );
    }

    #[test]
    fn trim_end() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello, world'
$string = $string.trimend(', world')
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("he".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.trimend('hed')
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("hello, worl".to_string())
        );

        let input = r#"
$string = 'hello, world '
$string = $string.trimend()
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("\"hello, world \".trimend()".to_string())
        );
        assert_eq!(
            script_res.errors()[0].to_string(),
            "MethodError: Incorrect arguments \"[]\" for method \"trim_end\"".to_string()
        );
    }

    #[test]
    fn trim_start() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello, world'
$string = $string.trimstart(', world')
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("hello, world".to_string())
        );

        let input = r#"
$string = 'hello, world'
$string = $string.trimstart('hed')
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("llo, world".to_string())
        );

        let input = r#"
$string = 'hello, world '
$string = $string.trimstart()
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("\"hello, world \".trimstart()".to_string())
        );
        assert_eq!(
            script_res.errors()[0].to_string(),
            "MethodError: Incorrect arguments \"[]\" for method \"trim_start\"".to_string()
        );
    }
}
