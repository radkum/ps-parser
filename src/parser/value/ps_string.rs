use std::{cmp::Ordering, sync::LazyLock};

use smart_default::SmartDefault;

use super::{MethodCallType, MethodError, MethodResult, RuntimeObject, Val, ValType};
use crate::parser::value::runtime_object::RuntimeResult;
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
        match name.to_ascii_lowercase().as_str() {
            "normalize" => Ok(normalize),
            _ => Err(MethodError::MethodNotFound(name.to_string()).into()),
        }
    }

    fn type_definition(&self) -> RuntimeResult<super::ValType> {
        Ok(ValType::String)
    }

    fn name(&self) -> String {
        ValType::String.name()
    }
}

fn normalize(object: Val, args: Vec<Val>) -> MethodResult<Val> {
    let Val::String(PsString(input)) = object else {
        return Err(MethodError::ObjectNotFound(object.cast_to_string()));
    };

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
        "FormKD" => input.nfkd().collect(),                        // Compatibility Decomposition
        "FormKC" => input.nfkc().collect(),                        // Compatibility Composition
        _ => input.to_string(),                                    // Default: no normalization
    };
    Ok(Val::String(res.into()))
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
