use super::{
    MethodError, MethodResult, PsString, RuntimeTypeTrait, StaticFnCallType, Val,
    runtime_object::RuntimeResult, val_type::ObjectType,
};
#[derive(Debug, Clone)]
pub(crate) struct Convert {}

impl RuntimeTypeTrait for Convert {
    fn static_method(&self, name: &str) -> RuntimeResult<StaticFnCallType> {
        match name.to_ascii_lowercase().as_str() {
            "frombase64string" => Ok(Box::new(from_base_64_string)),
            _ => Err(MethodError::MethodNotFound(name.to_string()).into()),
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

fn from_base_64_string(args: Vec<Val>) -> MethodResult<Val> {
    use base64::prelude::*;

    if args.len() != 1 {
        //something wrong
        return Err(MethodError::new_incorrect_args("FromBase64String", args));
    }

    let arg = args[0].clone();
    let Val::String(PsString(s)) = arg else {
        return Err(MethodError::new_incorrect_args("FromBase64String", args));
    };

    let x = BASE64_STANDARD
        .decode(s)
        .map_err(|e| MethodError::RuntimeError(e.to_string()))?;

    Ok(Val::Array(x.iter().map(|b| Val::Char(*b as u32)).collect()))
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
    fn test_builtint_objects() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.parse_input(r#" [system.convert].name "#)
                .unwrap()
                .result(),
            PsValue::String("Convert".into())
        );
        assert_eq!(
            p.parse_input(r#" [system.convert] "#).unwrap().result(),
            PsValue::String(
                "IsPublic\tIsSerial\tName\tBaseType\n--------\t--------\t----\t--------\n    \
                 true\t    true\tConvert\tSystem.Object"
                    .into()
            )
        );
        let script_res = p.parse_input(r#" [system.convert]0 "#).unwrap();
        assert_eq!(script_res.result(), PsValue::Null);
        assert_eq!(
            script_res.errors()[0].to_string(),
            String::from("ValError: Failed to convert value Int32 to type RuntimeObject")
        );
    }
}
