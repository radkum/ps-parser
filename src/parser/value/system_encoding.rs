use super::{
    MethodError, MethodResult, PsString, RuntimeObjectTrait, RuntimeTypeTrait, Val, ValError,
    runtime_object::{MethodCallType, RuntimeError, RuntimeResult},
    val_type::ObjectType,
};
use crate::parser::MethodName;

#[derive(Debug, Clone)]
pub(crate) struct Encoding {}

impl RuntimeTypeTrait for Encoding {
    fn readonly_static_member(&self, name: &str) -> RuntimeResult<Val> {
        log::debug!("get_static_member called with name: {}", name);
        match name.to_ascii_lowercase().as_str() {
            "unicode" => Ok(Val::RuntimeObject(Box::new(UnicodeEncoding {}))),
            "utf8" => Ok(Val::RuntimeObject(Box::new(Utf8Encoding {}))),
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
    fn method(&self, method_name: MethodName) -> RuntimeResult<MethodCallType> {
        match method_name.name() {
            "getstring" => Ok(Box::new(get_string)),
            "getbytes" => Ok(Box::new(get_bytes)),
            _ => Err(MethodError::MethodNotFound(method_name.name().to_string()).into()),
        }
    }
    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait> {
        Box::new(self.clone())
    }
    fn type_definition(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(Encoding {})
    }
}

impl std::fmt::Display for UnicodeEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "System.Text.UnicodeEncoding")
    }
}

#[derive(Debug, Clone)]
struct Utf8Encoding {}

impl RuntimeObjectTrait for Utf8Encoding {
    fn method(&self, method_name: MethodName) -> RuntimeResult<MethodCallType> {
        match method_name.name() {
            "getstring" => Ok(Box::new(utf8_get_string)),
            _ => Err(MethodError::MethodNotFound(method_name.name().to_string()).into()),
        }
    }
    fn clone_rt(&self) -> Box<dyn RuntimeObjectTrait> {
        Box::new(self.clone())
    }
    fn type_definition(&self) -> Box<dyn RuntimeTypeTrait> {
        Box::new(Encoding {})
    }
}

impl std::fmt::Display for Utf8Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "System.Text.UTF8Encoding")
    }
}

fn get_bytes(_: &mut Val, args: Vec<Val>) -> MethodResult<Val> {
    let [Val::String(PsString(s))] = args.as_slice() else {
        return Err(MethodError::new_incorrect_args("getbytes", args));
    };
    let len = s.len().saturating_mul(2);
    if len > super::MAX_ARRAY_LEN || !crate::parser::charge(len.saturating_mul(size_of::<Val>())) {
        return Err(ValError::ArgumentOutOfRange("getbytes".to_string(), len as i64).into());
    }
    Ok(Val::Array(
        s.encode_utf16()
            .flat_map(u16::to_le_bytes)
            .map(|b| Val::Char(b as u32))
            .collect(),
    ))
}

fn utf8_get_string(_: &mut Val, args: Vec<Val>) -> MethodResult<Val> {
    let bytes = bytes_arg("getstring", args)?;
    Ok(Val::String(
        String::from_utf8_lossy(&bytes).into_owned().into(),
    ))
}

fn get_string(_: &mut Val, args: Vec<Val>) -> MethodResult<Val> {
    Ok(Val::String(
        string_from_vec(bytes_arg("getstring", args)?).into(),
    ))
}

pub(crate) fn bytes_arg(method: &str, args: Vec<Val>) -> MethodResult<Vec<u8>> {
    use crate::parser::ValType;
    let arg = if args.len() == 1 {
        args[0].to_owned()
    } else {
        Val::Array(args)
    };

    let Ok(chars) = arg.cast_to_typed_array(Some(Box::new(ValType::Char))) else {
        return Err(MethodError::new_incorrect_args(method, arg.cast_to_array()));
    };

    chars
        .iter()
        .map(|v| {
            let Val::Char(c) = v else {
                return Err(MethodError::new_incorrect_args(method, arg.cast_to_array()));
            };
            Ok(*c as u8)
        })
        .collect()
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
            p.parse_script(r#" [system.text.encoding]::unicode "#)
                .unwrap()
                .result(),
            PsValue::String("System.Text.UnicodeEncoding".into())
        );
        assert_eq!(
            p.parse_script(r#" [system.text.encoding]"#)
                .unwrap()
                .result(),
            PsValue::String(
                "IsPublic\tIsSerial\tName\tBaseType\n--------\t--------\t----\t--------\n    \
                 true\t    true\tEncoding\tSystem.Object"
                    .into()
            )
        );
        assert_eq!(
            p.parse_script(r#" [system.text.encoding].name"#)
                .unwrap()
                .result(),
            PsValue::String("Encoding".into())
        );
        assert_eq!(
            p.parse_script(r#" [system.text.encoding].basetype.name"#)
                .unwrap()
                .result(),
            PsValue::String("System.Object".into())
        );
        assert_eq!(
            p.parse_script(r#" [system.text.encoding]"adsf" "#)
                .unwrap()
                .result(),
            PsValue::Null
        );
        assert_eq!(
            p.parse_script(r#" [system.text.encoding]"adsf" "#)
                .unwrap()
                .errors()[0]
                .to_string(),
            "ValError: Failed to convert value String to type RuntimeObject".to_string()
        );
    }

    #[test]
    fn utf8_and_unicode_bytes_round_trip() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.parse_script(r#" [Text.Encoding]::UTF8.GetString([byte[]](0x41, 0xc3, 0xb3)) "#)
                .unwrap()
                .result(),
            PsValue::String("Aó".into())
        );
        assert_eq!(
            p.parse_script(
                r#" [Text.Encoding]::Unicode.GetString([Text.Encoding]::Unicode.GetBytes("Aó")) "#
            )
            .unwrap()
            .result(),
            PsValue::String("Aó".into())
        );
    }
}
