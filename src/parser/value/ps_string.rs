use std::{cmp::Ordering, sync::LazyLock};

use smart_default::SmartDefault;

use super::{MethodCallType, MethodError, MethodResult, RuntimeObject, Val, ValType};
use crate::parser::value::{RuntimeError, runtime_object::RuntimeResult};
#[derive(Clone, Debug, SmartDefault, PartialEq)]
pub(crate) struct PsString(pub String);

impl From<&str> for PsString {
    fn from(s: &str) -> Self {
        PsString(s.to_string())
    }
}

impl From<String> for PsString {
    fn from(s: String) -> Self {
        PsString(s)
    }
}

impl RuntimeObject for PsString {
    fn method(&self, name: &str) -> RuntimeResult<MethodCallType> {
        let name = name.to_ascii_lowercase();
        let fn_ptr = match name.to_ascii_lowercase().as_str() {
            "normalize" => Self::normalize,
            "replace" => Self::replace,
            "substring" => Self::substring,
            "remove" => Self::remove,
            // "clone" => Ok(clone),
            // "copyto" => Ok(remove),
            // "isnormalized" => Ok(remove),
            // "split" => Ok(remove),
            // "tostring" => Ok(remove),
            // "touper" => Ok(remove),
            // "touperinvariant" => Ok(remove),
            // "tolower" => Ok(remove),
            // "tolowerinvariant" => Ok(remove),
            _ => Err(RuntimeError::MethodNotFound(name.to_string()))?,
        };

        Ok(Box::new(move |v: &Val, args: Vec<Val>| {
            if let Val::String(str) = v {
                fn_ptr(str, args)
            } else {
                Err(MethodError::ObjectNotFound(v.cast_to_string()))
            }
        }))
    }

    fn type_definition(&self) -> RuntimeResult<super::ValType> {
        Ok(ValType::String)
    }

    fn name(&self) -> String {
        ValType::String.name()
    }
}

impl PsString {
    fn normalize(&self, args: Vec<Val>) -> MethodResult<Val> {
        let PsString(input) = self;

        if args.len() != 1 {
            //something wrong
            return Err(MethodError::new_incorrect_args("FromBase64String", args));
        }

        let arg = args[0].clone();
        let Val::String(PsString(form)) = arg else {
            return Err(MethodError::new_incorrect_args("FromBase64String", args));
        };

        use unicode_normalization::UnicodeNormalization;

        let res = match form.as_str() {
            "FormD" => input.nfd().filter(|c| c.is_ascii()).collect(), // Canonical Decomposition
            "FormC" => input.nfc().collect(),                          // Canonical Composition
            "FormKD" => input.nfkd().collect(),                        /* Compatibility */
            // Decomposition
            "FormKC" => input.nfkc().collect(), // Compatibility Composition
            _ => input.to_string(),             // Default: no normalization
        };
        Ok(Val::String(res.into()))
    }

    fn replace(&self, args: Vec<Val>) -> MethodResult<Val> {
        let PsString(input) = self;

        if args.len() != 2 {
            //something wrong
            return Err(MethodError::new_incorrect_args("Replace", args));
        }

        if !matches!(args[0], Val::String(_) | Val::Char(_)) {
            return Err(MethodError::new_incorrect_args("Replace", args));
        }

        if !matches!(args[1], Val::String(_) | Val::Char(_)) {
            return Err(MethodError::new_incorrect_args("Replace", args));
        }

        let old = args[0].cast_to_string();
        let new = args[1].cast_to_string();
        let res = input.replace(&old, &new);
        Ok(Val::String(PsString(res)))
    }

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
        return Ok((start_index, end_index));
    }

    fn substring(&self, args: Vec<Val>) -> MethodResult<Val> {
        //string Substring(int startIndex)
        //string Substring(int startIndex, int length)
        let PsString(input) = self;
        let (start_index, end_index) = self.args_for_remove_and_substring(args, "Substring")?;
        let res = input[start_index..end_index].to_string();
        return Ok(Val::String(PsString(res)));
    }

    fn remove(&self, args: Vec<Val>) -> MethodResult<Val> {
        //string Remove(int startIndex, int count)
        //string Remove(int startIndex)
        let PsString(input) = self;
        let (start_index, end_index) = self.args_for_remove_and_substring(args, "Remove")?;
        let res = input[..start_index].to_string() + &input[end_index..];
        return Ok(Val::String(PsString(res)));
    }
}

// very strange. En-us culture has different ordering than default. A (ascii 65)
// is greater than a(ascii 97 need to Collator object to perform string
// comparison
#[cfg(feature = "en-us")]
const COLLATOR: LazyLock<icu::collator::Collator> = LazyLock::new(|| {
    icu::collator::Collator::try_new(
        &icu::locid::locale!("en-US").into(),
        icu::collator::CollatorOptions::new(),
    )
    .unwrap()
});

pub fn str_cmp(s1: &str, s2: &str, case_insensitive: bool) -> Ordering {
    if case_insensitive {
        s1.to_ascii_lowercase().cmp(&s2.to_ascii_lowercase())
    } else if cfg!(feature = "en-us") {
        COLLATOR.compare(s1, s2)
    } else {
        s1.cmp(s2)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn replace() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello, world'
$string = $string.replace('he','e')
$string = $string.replace('ll','l')
$string = $string.replace('o,','o')
$string = $string.replace(' ','.')
$string = $string.replace('wo','d')
$string = $string.replace('rld','ll');$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("elo.dll".to_string()));
    }

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
