use std::{cmp::Ordering, sync::LazyLock};

use smart_default::SmartDefault;
mod normalize;
mod pad;
mod substring;
mod to_upper_lower;
mod trim;
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
            "clone" => Self::_clone,
            // "copyto" => Self::copyto,
            // "tocharsarray" => Self::tocharsarray,
            "isnormalized" => Self::is_normalized,
            "split" => Self::split,
            "tostring" => Self::_clone,
            "toupper" => Self::to_upper,
            "toupperinvariant" => Self::to_upper_invariant,
            "tolower" => Self::to_lower,
            "tolowerinvariant" => Self::to_lower_invariant,
            "insert" => Self::insert,
            "padleft" => Self::pad_left,
            "padright" => Self::pad_right,
            "trim" => Self::trim,
            "trimend" => Self::trim_end,
            "trimstart" => Self::trim_start,
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
    fn _clone(&self, args: Vec<Val>) -> MethodResult<Val> {
        if !args.is_empty() {
            return Err(MethodError::new_incorrect_args("Clone", args));
        }
        Ok(Val::String(self.clone()))
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

    fn insert(&self, args: Vec<Val>) -> MethodResult<Val> {
        let PsString(mut input) = self.clone();

        if args.len() != 2 {
            //something wrong
            return Err(MethodError::new_incorrect_args("Insert", args));
        }

        let Val::Int(idx) = args[0] else {
            return Err(MethodError::new_incorrect_args("Insert", args));
        };

        let value = if args[1].ttype() == ValType::String || args[1].ttype() == ValType::Char {
            args[1].cast_to_string()
        } else {
            Err(MethodError::new_incorrect_args("Insert", args))?
        };

        input.insert_str(idx as usize, value.as_str());
        Ok(Val::String(PsString(input)))
    }

    fn split(&self, args: Vec<Val>) -> MethodResult<Val> {
        let PsString(mut input) = self.clone();

        let args_len = args.len();
        if args_len != 1 && args_len != 2 {
            //something wrong
            return Err(MethodError::new_incorrect_args("Split", args.clone()));
        }

        let arg_1 = args[0].to_owned();

        let value = if arg_1.ttype() == ValType::String || arg_1.ttype() == ValType::Char {
            arg_1.cast_to_string()
        } else {
            Err(MethodError::new_incorrect_args("Split", args.clone()))?
        };

        let parts = if args_len == 2
            && let Val::Int(idx) = args[1]
        {
            let mut parts = vec![];
            if idx == 0 {
                return Ok(Val::Array(vec![]));
            }
            for _ in 0..idx - 1 {
                if let Some((before, after)) = input.split_once(value.as_str()) {
                    parts.push(before.to_string());
                    input = after.to_string();
                } else {
                    break;
                }
            }
            parts.push(input);
            parts
        } else {
            input
                .split(value.as_str())
                .map(String::from)
                .collect::<Vec<String>>()
        };
        let parts = parts
            .into_iter()
            .map(|part| Val::String(part.into()))
            .collect();
        Ok(Val::Array(parts))
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
    fn insert() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello'
$string = $string.insert(1,'r')
$string = $string.insert(4,"dll")
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::String("hreldlllo".to_string())
        );
    }

    #[test]
    fn split() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'hello world'
$string = $string.split('l')
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::String("he".to_string()),
                PsValue::String("".to_string()),
                PsValue::String("o wor".to_string()),
                PsValue::String("d".to_string()),
            ])
        );

        let input = r#"
$string = 'hello world'
$string = $string.split('l', 2)
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![
                PsValue::String("he".to_string()),
                PsValue::String("lo world".to_string()),
            ])
        );

        let input = r#"
$string = 'hello world'
$string = $string.split('z', 2)
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(
            script_res.result(),
            PsValue::Array(vec![PsValue::String("hello world".to_string()),])
        );

        let input = r#"
$string = 'hello world'
$string = $string.split('z', 0)
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::Array(vec![]));
    }
}
