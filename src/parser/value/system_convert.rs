use super::{
    MethodError, MethodResult, PsString, RuntimeTypeTrait, StaticFnCallType, Val, ValError,
    runtime_object::RuntimeResult, val_type::ObjectType,
};
use crate::parser::MethodName;
#[derive(Debug, Clone)]
pub(crate) struct Convert {}

impl RuntimeTypeTrait for Convert {
    fn static_method(&self, name: MethodName) -> RuntimeResult<StaticFnCallType> {
        match name.name() {
            "frombase64string" => Ok(Box::new(from_base_64_string)),
            "tobase64string" => Ok(Box::new(to_base_64_string)),
            _ => Err(MethodError::MethodNotFound(name.name().into()).into()),
        }
    }
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(ObjectType {})
    }
    fn name(&self) -> String {
        "Convert".to_string()
    }

    fn full_name(&self) -> String {
        format!("System.{}", self.name())
    }

    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

fn from_base_64_string(mut args: Vec<Val>) -> MethodResult<Val> {
    use base64::prelude::*;

    if args.len() != 1 {
        //something wrong
        return Err(MethodError::new_incorrect_args("FromBase64String", args));
    }

    let s = match args.swap_remove(0) {
        Val::String(PsString(s)) => s,
        arg => {
            return Err(MethodError::new_incorrect_args(
                "FromBase64String",
                vec![arg],
            ));
        }
    };

    // the whitespace-free copy, the decoded bytes and their Vals
    let len = s.len().div_ceil(4).saturating_mul(3);
    if !crate::parser::fits(
        len.saturating_mul(size_of::<Val>() + 1)
            .saturating_add(s.len()),
    ) {
        return Err(
            ValError::ArgumentOutOfRange("FromBase64String".to_string(), len as i64).into(),
        );
    }
    // Convert.FromBase64String ignores whitespace
    let s = s.split_ascii_whitespace().collect::<String>();
    let x = BASE64_STANDARD
        .decode(s)
        .map_err(|e| MethodError::RuntimeError(e.to_string()))?;

    Ok(Val::Array(x.iter().map(|b| Val::Char(*b as u32)).collect()))
}

fn to_base_64_string(args: Vec<Val>) -> MethodResult<Val> {
    use base64::prelude::*;

    let bytes = super::system_encoding::bytes_arg("ToBase64String", args)?;
    Ok(Val::String(BASE64_STANDARD.encode(bytes).into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PowerShellSession, PsValue};
    #[test]
    fn test_from_base64_string() {
        let input = "SGVsbG8gV29ybGQh"; // "Hello World!"
        let args = vec![Val::String(PsString(input.to_string()))];
        let result = from_base_64_string(args).unwrap();

        if let Val::Array(arr) = result {
            let decoded: String = arr
                .iter()
                .map(|v| {
                    if let Val::Char(c) = v {
                        *c as u8 as char
                    } else {
                        '\0'
                    }
                })
                .collect();
            assert_eq!(decoded, "Hello World!");
        } else {
            panic!("Expected Val::Array");
        }
    }

    #[test]
    fn to_base64_string_without_the_system_prefix() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.parse_script(r#" [Convert]::ToBase64String([Convert]::FromBase64String("SGk=")) "#)
                .unwrap()
                .result(),
            PsValue::String("SGk=".into())
        );
    }

    #[test]
    fn to_base64_string_of_utf8_bytes() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.parse_script(
                r#" [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes('Get-PassHashes')) "#
            )
            .unwrap()
            .result(),
            PsValue::String("R2V0LVBhc3NIYXNoZXM=".into())
        );
    }

    #[test]
    fn from_base64_string_past_the_memory_budget_fails_before_decoding() {
        let decodes = |script: &str| {
            !PowerShellSession::new()
                .parse_script(script)
                .unwrap()
                .errors()
                .iter()
                .any(|e| e.to_string().contains("FromBase64String"))
        };
        assert!(decodes(
            "$b = [Convert]::FromBase64String('AAAA' * 400000); $b.Length"
        ));
        assert!(!decodes("[Convert]::FromBase64String('AAAA' * 4000000)"));
    }

    #[test]
    fn a_large_base64_blob_does_not_stop_the_rest_of_the_script() {
        // refused before decoding, without using up the budget
        let script = "[Convert]::FromBase64String('AAAA' * 4000000); & ('Get-Pass'+'Hashes')";
        let s = PowerShellSession::new().parse_script(script).unwrap();
        assert!(
            s.deobfuscated().contains("Get-PassHashes"),
            "{}",
            s.deobfuscated()
        );
    }

    #[test]
    fn from_base64_string_ignores_whitespace() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.parse_script("[Convert]::ToBase64String([Convert]::FromBase64String('QU JD'))")
                .unwrap()
                .result(),
            PsValue::String("QUJD".into())
        );
    }

    #[test]
    fn test_builtint_objects() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.parse_script(r#" [system.convert].name "#)
                .unwrap()
                .result(),
            PsValue::String("Convert".into())
        );
        assert_eq!(
            p.parse_script(r#" [system.convert] "#).unwrap().result(),
            PsValue::String(
                "IsPublic\tIsSerial\tName\tBaseType\n--------\t--------\t----\t--------\n    \
                 true\t    true\tConvert\tSystem.Object"
                    .into()
            )
        );
        let script_res = p.parse_script(r#" [system.convert]0 "#).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.errors()[0].to_string(),
            String::from("ValError: Failed to convert value Int32 to type RuntimeObject")
        );
    }
}
