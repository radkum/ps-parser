use super::{MethodError, MethodResult, PsString, Val};

impl PsString {
    pub(super) fn pad_left(&self, args: Vec<Val>) -> MethodResult<Val> {
        let PsString(mut input) = self.clone();

        if args.len() != 1 {
            //something wrong
            return Err(MethodError::new_incorrect_args("PadLeft", args));
        }

        let Val::Int(width) = args[0] else {
            return Err(MethodError::new_incorrect_args("PadLeft", args));
        };

        let padding = width.saturating_sub(input.len() as i64);
        if padding > 0 {
            input.insert_str(0, &" ".repeat(padding as usize));
        }

        Ok(Val::String(PsString(input)))
    }

    pub(super) fn pad_right(&self, args: Vec<Val>) -> MethodResult<Val> {
        let PsString(mut input) = self.clone();

        if args.len() != 1 {
            //something wrong
            return Err(MethodError::new_incorrect_args("PadRight", args));
        }

        let Val::Int(width) = args[0] else {
            return Err(MethodError::new_incorrect_args("PadRight", args));
        };

        let padding = width.saturating_sub(input.len() as i64);
        if padding > 0 {
            input.push_str(&" ".repeat(padding as usize));
        }

        Ok(Val::String(PsString(input)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn pad_left() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello'
$string = $string.padleft(2)
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("hello".to_string()));

        let input = r#"
$string = 'hello'
$string = $string.padleft(10)
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("     hello".to_string())
        );

        let input = r#"
$string = 'hello'
$string = $string.padleft()
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("\"hello\".padleft()".to_string())
        );
        assert_eq!(
            script_res.errors()[0].to_string(),
            "MethodError: Incorrect arguments \"[]\" for method \"PadLeft\"".to_string()
        );
    }

    #[test]
    fn pad_right() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello'
$string = $string.padright(10)
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("hello     ".to_string())
        );

        let input = r#"
$string = 'hello'
$string = $string.padright()
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("\"hello\".padright()".to_string())
        );
        assert_eq!(
            script_res.errors()[0].to_string(),
            "MethodError: Incorrect arguments \"[]\" for method \"PadRight\"".to_string()
        );
    }
}
