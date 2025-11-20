use super::{
    MethodError, MethodResult, RuntimeObjectTrait, RuntimeTypeTrait, Val,
    runtime_object::{MethodCallType, RuntimeError, RuntimeResult},
    val_type::ObjectType,
};

#[derive(Debug, Clone)]
pub(crate) struct Encoding {}

impl RuntimeTypeTrait for Encoding {
    fn readonly_static_member(&self, name: &str) -> RuntimeResult<Val> {
        log::debug!("get_static_member called with name: {}", name);
        match name.to_ascii_lowercase().as_str() {
            "unicode" => Ok(Val::RuntimeObject(Box::new(UnicodeEncoding {}))),
            _ => Err(RuntimeError::MemberNotFound(name.to_string())),
        }
    }
    fn base_type(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(ObjectType {})
    }
    fn name(&self) -> String {
        "Encoding".to_string()
    }
    fn full_name(&self) -> String {
        format!("System.Text.{}", self.name())
    }
    fn clone_rt(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct UnicodeEncoding {}

impl RuntimeObjectTrait for UnicodeEncoding {
    fn method(&self, name: &str) -> RuntimeResult<MethodCallType> {
        match name.to_ascii_lowercase().as_str() {
            "getstring" => Ok(Box::new(get_string)),
            _ => Err(MethodError::MethodNotFound(name.to_string()).into()),
        }
    }
    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait> {
        Box::new(self.clone())
    }
}

impl std::fmt::Display for UnicodeEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "System.Text.UnicodeEncoding")
    }
}

fn get_string(_: &Val, args: Vec<Val>) -> MethodResult<Val> {
    use crate::parser::ValType;
    let arg = if args.len() == 1 {
        args[0].to_owned()
    } else {
        Val::Array(args)
    };

    let Ok(chars) = arg.cast_to_typed_array(Some(Box::new(ValType::Char))) else {
        return Err(MethodError::new_incorrect_args(
            "getstring",
            arg.cast_to_array(),
        ));
    };

    let v = chars
        .iter()
        .map(|v| {
            let Val::Char(c) = v else {
                return Err(MethodError::new_incorrect_args(
                    "getstring",
                    arg.cast_to_array(),
                ));
            };
            Ok(*c as u8)
        })
        .collect::<Result<Vec<u8>, _>>()?;

    Ok(Val::String(string_from_vec(v).into()))
}

fn string_from_vec(mut buf: Vec<u8>) -> String {
    //if buf len is odd, then last char should be 0x65533
    let add_last = if !buf.len().is_multiple_of(2) {
        buf.pop();
        true
    } else {
        false
    };
    let u16_buffer = unsafe { buf.align_to_mut::<u16>().1 };

    let mut ends_with_null = false;
    if let Some(c) = u16_buffer.last()
        && *c == 0
    {
        ends_with_null = true;
    }

    let mut res_string = String::from_utf16_lossy(u16_buffer);
    if ends_with_null {
        res_string.pop();
    }

    if add_last {
        res_string.push('\u{FFFD}');
    }
    res_string
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellSession, PsValue};

    #[test]
    fn test_builtint_objects() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.parse_input(r#" [system.text.encoding]::unicode "#)
                .unwrap()
                .result(),
            PsValue::String("System.Text.UnicodeEncoding".into())
        );
        assert_eq!(
            p.parse_input(r#" [system.text.encoding]"#)
                .unwrap()
                .result(),
            PsValue::String(
                "IsPublic\tIsSerial\tName\tBaseType\n--------\t--------\t----\t--------\n    \
                 true\t    true\tEncoding\tSystem.Object"
                    .into()
            )
        );
        assert_eq!(
            p.parse_input(r#" [system.text.encoding].name"#)
                .unwrap()
                .result(),
            PsValue::String("Encoding".into())
        );
        assert_eq!(
            p.parse_input(r#" [system.text.encoding].basetype.name"#)
                .unwrap()
                .result(),
            PsValue::String("System.Object".into())
        );
        assert_eq!(
            p.parse_input(r#" [system.text.encoding]"adsf" "#)
                .unwrap()
                .result(),
            PsValue::Null
        );
        assert_eq!(
            p.parse_input(r#" [system.text.encoding]"adsf" "#)
                .unwrap()
                .errors()[0]
                .to_string(),
            "ValError: Failed to convert value String to type RuntimeObject".to_string()
        );
    }
}
