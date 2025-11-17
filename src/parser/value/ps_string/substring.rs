use super::{MethodError, MethodResult, PsString, Val};

impl PsString {
    fn args_for_remove_and_substring(
        &self,
        args: Vec<Val>,
        fn_name: &str,
    ) -> MethodResult<(usize, usize)> {
        let PsString(input) = self;

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
            if start_index + length > input.len() {
                return Err(MethodError::Exception(format!(
                    "Exception calling \"{}\" with \"2\" argument(s): \"Index and length must \
                     refer to a location within the string. Parameter name: length\"",
                    fn_name
                )));
            }
            length
        } else {
            input.len()
        };

        if start_index > input.len() {
            return Err(MethodError::Exception(format!(
                "Exception calling \"{}\" with \"1\" argument(s): \"startIndex cannot be larger \
                 than length of string. Parameter name: startIndex\"",
                fn_name
            )));
        }

        let end_index = std::cmp::min(start_index + length, input.len());
        Ok((start_index, end_index))
    }

    pub(super) fn substring(&self, args: Vec<Val>) -> MethodResult<Val> {
        //string Substring(int startIndex)
        //string Substring(int startIndex, int length)
        let PsString(input) = self;
        let (start_index, end_index) = self.args_for_remove_and_substring(args, "Substring")?;
        let res = input[start_index..end_index].to_string();
        Ok(Val::String(PsString(res)))
    }

    pub(super) fn remove(&self, args: Vec<Val>) -> MethodResult<Val> {
        //string Remove(int startIndex, int count)
        //string Remove(int startIndex)
        let PsString(input) = self;
        let (start_index, end_index) = self.args_for_remove_and_substring(args, "Remove")?;
        let res = input[..start_index].to_string() + &input[end_index..];
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
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("ello".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.substring(7);$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("world".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.substring(7,5);$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("world".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.substring(7,6);$string"#;
        let script_res = p.parse_input(input).unwrap();
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
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.substring(13);$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.errors()[0].to_string(),
            "MethodError: Exception calling \"Substring\" with \"1\" argument(s): \"startIndex \
             cannot be larger than length of string. Parameter name: startIndex\""
                .to_string()
        );

        let input = r#"
$string = 'hello, world'
$string = $string.substring(5,0);$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("".to_string()));
    }

    #[test]
    fn remove() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello, world'
$string = $string.remove(1, 4);$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("h, world".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.remove(7);$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("hello, ".to_string()));

        let input = r#"
$string = 'hello, world'
$string = $string.remove(7,15);$string"#;
        let script_res = p.parse_input(input).unwrap();
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
}
