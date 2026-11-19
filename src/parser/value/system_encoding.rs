use super::{
    MethodError, MethodResult, PsString, RuntimeObjectTrait, RuntimeTypeTrait, StaticFnCallType,
    Val, ValError,
    runtime_object::{MethodCallType, RuntimeError, RuntimeResult},
    val_type::ObjectType,
};
use crate::parser::MethodName;

#[derive(Debug, Clone)]
pub(crate) struct Encoding {}

impl RuntimeTypeTrait for Encoding {
    fn readonly_static_member(&self, name: &str) -> RuntimeResult<Val> {
        log::debug!("get_static_member called with name: {}", name);
        encoding(&name.to_ascii_lowercase())
            .ok_or_else(|| RuntimeError::MemberNotFound(name.to_string()))
    }
    fn static_method(&self, name: MethodName) -> RuntimeResult<StaticFnCallType> {
        match name.name() {
            "getencoding" => Ok(Box::new(get_encoding)),
            _ => Err(MethodError::MethodNotFound(name.name().into()).into()),
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
            "getbytes" => Ok(Box::new(|_, args| get_bytes(Kind::Unicode, args))),
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

// Default is UTF-8 as in PowerShell 7; Windows PowerShell uses ANSI
fn encoding(name: &str) -> Option<Val> {
    let kind = match name {
        "unicode" | "utf-16" | "utf-16le" => {
            return Some(Val::RuntimeObject(Box::new(UnicodeEncoding {})));
        }
        "utf8" | "utf-8" | "default" => Kind::Utf8,
        "ascii" | "us-ascii" => Kind::Ascii,
        "bigendianunicode" | "utf-16be" | "unicodefffe" => Kind::BigEndianUnicode,
        "utf32" | "utf-32" | "utf-32le" => Kind::Utf32,
        _ => return None,
    };
    Some(Val::RuntimeObject(Box::new(TextEncoding(kind))))
}

fn get_encoding(args: Vec<Val>) -> MethodResult<Val> {
    let name = match args.as_slice() {
        [Val::Int(65001)] => "utf-8".to_string(),
        [Val::Int(1200)] => "utf-16".to_string(),
        [Val::Int(1201)] => "utf-16be".to_string(),
        [Val::Int(12000)] => "utf-32".to_string(),
        [Val::Int(20127)] => "us-ascii".to_string(),
        [Val::String(PsString(s))] => s.to_ascii_lowercase(),
        _ => String::new(),
    };
    encoding(&name).ok_or_else(|| MethodError::new_incorrect_args("GetEncoding", args))
}

#[derive(Debug, Clone, Copy)]
enum Kind {
    Utf8,
    Ascii,
    Unicode,
    BigEndianUnicode,
    Utf32,
}

#[derive(Debug, Clone)]
struct TextEncoding(Kind);

impl RuntimeObjectTrait for TextEncoding {
    fn method(&self, method_name: MethodName) -> RuntimeResult<MethodCallType> {
        let kind = self.0;
        match method_name.name() {
            "getstring" => Ok(Box::new(move |_, args| {
                let bytes = bytes_arg("getstring", args)?;
                Ok(Val::String(decode(kind, bytes).into()))
            })),
            "getbytes" => Ok(Box::new(move |_, args| get_bytes(kind, args))),
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

impl std::fmt::Display for TextEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self.0 {
            Kind::Utf8 => "UTF8Encoding",
            Kind::Ascii => "ASCIIEncoding",
            Kind::Unicode | Kind::BigEndianUnicode => "UnicodeEncoding",
            Kind::Utf32 => "UTF32Encoding",
        };
        write!(f, "System.Text.{name}")
    }
}

fn get_bytes(kind: Kind, args: Vec<Val>) -> MethodResult<Val> {
    let [Val::String(PsString(s))] = args.as_slice() else {
        return Err(MethodError::new_incorrect_args("getbytes", args));
    };
    let factor = match kind {
        Kind::Utf8 | Kind::Ascii => 1,
        Kind::Unicode | Kind::BigEndianUnicode => 2,
        Kind::Utf32 => 4,
    };
    let len = s.len().saturating_mul(factor);
    if len > super::MAX_ARRAY_LEN || !crate::parser::charge(len.saturating_mul(size_of::<Val>())) {
        return Err(ValError::ArgumentOutOfRange("getbytes".to_string(), len as i64).into());
    }
    let bytes: Vec<u8> = match kind {
        Kind::Utf8 => s.as_bytes().to_vec(),
        Kind::Ascii => s
            .chars()
            .map(|c| if c.is_ascii() { c as u8 } else { b'?' })
            .collect(),
        Kind::Unicode => s.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        Kind::BigEndianUnicode => s.encode_utf16().flat_map(u16::to_be_bytes).collect(),
        Kind::Utf32 => s.chars().flat_map(|c| (c as u32).to_le_bytes()).collect(),
    };
    Ok(Val::Array(
        bytes.into_iter().map(|b| Val::Char(b as u32)).collect(),
    ))
}

fn decode(kind: Kind, bytes: Vec<u8>) -> String {
    let tail = |n: usize| {
        if bytes.len().is_multiple_of(n) {
            ""
        } else {
            "\u{FFFD}"
        }
    };
    match kind {
        Kind::Utf8 => String::from_utf8_lossy(&bytes).into_owned(),
        Kind::Ascii => bytes
            .iter()
            .map(|&b| if b.is_ascii() { b as char } else { '?' })
            .collect(),
        Kind::Unicode => string_from_vec(bytes),
        Kind::BigEndianUnicode => {
            let units = bytes
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]));
            char::decode_utf16(units)
                .map(|c| c.unwrap_or('\u{FFFD}'))
                .collect::<String>()
                + tail(2)
        }
        Kind::Utf32 => {
            bytes
                .chunks_exact(4)
                .map(|c| {
                    char::from_u32(u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                        .unwrap_or('\u{FFFD}')
                })
                .collect::<String>()
                + tail(4)
        }
    }
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

    #[test]
    fn every_encoding_round_trips_its_bytes() {
        for (encoding, base64, decoded) in [
            ("[Text.Encoding]::UTF8", "QcOz4oKs", "Aó€"),
            ("[Text.Encoding]::Default", "QcOz4oKs", "Aó€"),
            ("[Text.Encoding]::Unicode", "QQDzAKwg", "Aó€"),
            ("[Text.Encoding]::ASCII", "QT8/", "A??"),
            ("[Text.Encoding]::BigEndianUnicode", "AEEA8yCs", "Aó€"),
            ("[Text.Encoding]::UTF32", "QQAAAPMAAACsIAAA", "Aó€"),
            (
                "[System.Text.Encoding]::GetEncoding('utf-8')",
                "QcOz4oKs",
                "Aó€",
            ),
            (
                "[System.Text.Encoding]::GetEncoding(1201)",
                "AEEA8yCs",
                "Aó€",
            ),
        ] {
            let mut p = PowerShellSession::new();
            let script = format!("[Convert]::ToBase64String({encoding}.GetBytes(\"Aó€\"))");
            let result = p.parse_script(&script).unwrap().result();
            assert_eq!(result, PsValue::String(base64.into()), "{script}");
            let script = format!("{encoding}.GetString([Convert]::FromBase64String('{base64}'))");
            let result = p.parse_script(&script).unwrap().result();
            assert_eq!(result, PsValue::String(decoded.into()), "{script}");
        }
    }

    #[test]
    fn a_truncated_multibyte_sequence_decodes_to_a_replacement_char() {
        let mut p = PowerShellSession::new();
        for (encoding, base64) in [("BigEndianUnicode", "AEEA"), ("UTF32", "QQAAAAA=")] {
            let script = format!(
                "[Text.Encoding]::{encoding}.GetString([Convert]::FromBase64String('{base64}'))"
            );
            let result = p.parse_script(&script).unwrap().result();
            assert_eq!(result, PsValue::String("A\u{FFFD}".into()), "{script}");
        }
    }

    #[test]
    fn get_bytes_past_the_array_limit_fails() {
        let mut p = PowerShellSession::new();
        let s = p
            .parse_script("[Text.Encoding]::UTF32.GetBytes('a' * 300000)")
            .unwrap();
        assert!(
            s.errors()
                .iter()
                .any(|e| e.to_string().contains("getbytes")),
            "{:?}",
            s.errors()
        );
    }
}
