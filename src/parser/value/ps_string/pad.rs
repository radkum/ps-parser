use super::{MethodError, MethodResult, PsString, Val};

impl PsString {
    pub(super) fn normalize(&self, args: Vec<Val>) -> MethodResult<Val> {
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

    pub(super) fn is_normalized(&self, _args: Vec<Val>) -> MethodResult<Val> {
        Ok(Val::Bool(true))
    }
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn normalize() {
        let mut p = PowerShellSession::new();
        let input = r#"
$string = 'Âmí'+'Ùtìl'
$string = $string.normalize("FormD")
$string"#;
        let script_res = p.parse_input(input).unwrap();
        assert_eq!(script_res.result(), PsValue::String("AmiUtil".to_string()));
    }
}
