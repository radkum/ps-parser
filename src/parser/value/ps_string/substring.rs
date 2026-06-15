use super::{MethodError, MethodResult, PsString, Val};

impl PsString {
    /// Validates arguments and returns (start_char_index, length_in_chars).
    /// Uses character counts (not byte counts) to match PowerShell semantics.
    fn args_for_remove_and_substring(
        &self,
        args: Vec<Val>,
        fn_name: &str,
    ) -> MethodResult<(usize, usize)> {
        let PsString(input) = self;
        let char_count = input.chars().count();

        if args.len() != 2 && args.len() != 1 {
            //something wrong
            return Err(MethodError::new_incorrect_args(fn_name, args));
        }

        if !matches!(args[0], Val::Int(_)) {
            return Err(MethodError::new_incorrect_args(fn_name, args));
        }
        let start_index = args[0].cast_to_int()? as usize;

        // substring is overloaded method. It can take 1 or 2 arguments. Second argument
        // is optional
        let length = if args.len() == 2 {
            if !matches!(args[1], Val::Int(_)) {
                return Err(MethodError::new_incorrect_args(fn_name, args));
            }

            let length = args[1].cast_to_int()? as usize;
            if start_index + length > char_count {
                return Err(MethodError::Exception(format!(
                    "Exception calling \"{}\" with \"2\" argument(s): \"Index and length must \
                     refer to a location within the string. Parameter name: length\"",
                    fn_name
                )));
            }
            length
        } else {
            char_count.saturating_sub(start_index)
        };

        if start_index > char_count {
            return Err(MethodError::Exception(format!(
                "Exception calling \"{}\" with \"1\" argument(s): \"startIndex cannot be larger \
                 than length of string. Parameter name: startIndex\"",
                fn_name
            )));
        }

        Ok((start_index, length))
    }

    pub(super) fn substring(&self, args: Vec<Val>) -> MethodResult<Val> {
        //string Substring(int startIndex)
        //string Substring(int startIndex, int length)
        let PsString(input) = self;
        let (start_index, length) = self.args_for_remove_and_substring(args, "Substring")?;
        let res: String = input.chars().skip(start_index).take(length).collect();
        Ok(Val::String(PsString(res)))
    }

    pub(super) fn remove(&self, args: Vec<Val>) -> MethodResult<Val> {
        //string Remove(int startIndex, int count)
        //string Remove(int startIndex)
        let PsString(input) = self;
        let (start_index, length) = self.args_for_remove_and_substring(args, "Remove")?;
        let prefix: String = input.chars().take(start_index).collect();
        let suffix: String = input.chars().skip(start_index + length).collect();
        let res = prefix + &suffix;
        Ok(Val::String(PsString(res)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn substring() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello, world'
$string = $string.substring(1, 4);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("ello".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.substring(7);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("world".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.substring(7,5);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("world".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.substring(7,6);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(
            script_res.errors()[0].to_string(),
            "MethodError: Exception calling \"Substring\" with \"2\" argument(s): \"Index and \
             length must refer to a location within the string. Parameter name: length\""
                .to_string()
        );
        assert_eq!(
            script_res.result(),
            PsValue::String(r#""hello, world".substring(7, 6)"#.to_string())
        );

        let input = r#"
$string = 'hello, world'
$string = $string.substring(12);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.substring(13);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(
            script_res.errors()[0].to_string(),
            "MethodError: Exception calling \"Substring\" with \"1\" argument(s): \"startIndex \
             cannot be larger than length of string. Parameter name: startIndex\""
                .to_string()
        );

        let input = r#"
$string = 'hello, world'
$string = $string.substring(5,0);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("".to_string()));
    }

    #[test]
    fn remove() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello, world'
$string = $string.remove(1, 4);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("h, world".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.remove(7);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("hello, ".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.remove(7,15);$string"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("\"hello, world\".remove(7, 15)".to_string())
        );
        assert_eq!(
            script_res.errors()[0].to_string(),
            "MethodError: Exception calling \"Remove\" with \"2\" argument(s): \"Index and length \
             must refer to a location within the string. Parameter name: length\""
                .to_string()
        );
    }

    #[test]
    fn substring_utf8() {
        let mut p = PowerShellSession::new();
        // "żółć" has 4 characters but 8 bytes in UTF-8
        let input = r#"
$string = 'żółć'
$string.substring(1, 2)"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("ół".to_string()));

        let input = r#"
$string = 'żółć'
$string.substring(2)"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("łć".to_string()));

        // Mixed ASCII and non-ASCII
        let input = r#"
$string = 'ażółćb'
$string.substring(1, 4)"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("żółć".to_string()));
    }

    #[test]
    fn remove_utf8() {
        let mut p = PowerShellSession::new();
        // "żółć" has 4 characters but 8 bytes in UTF-8
        let input = r#"
$string = 'żółć'
$string.remove(1, 2)"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("żć".to_string()));

        let input = r#"
$string = 'żółć'
$string.remove(2)"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("żó".to_string()));

        // Mixed ASCII and non-ASCII
        let input = r#"
$string = 'ażółćb'
$string.remove(1, 4)"#;
        let script_res = p.parse_script(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("ab".to_string()));
    }
}
