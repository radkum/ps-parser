use super::{
    MethodError, MethodResult, RuntimeObject, Val,
    runtime_object::{MethodCallType, RuntimeError, RuntimeResult},
};

#[derive(Debug, Clone)]
pub(crate) struct Encoding {}

impl RuntimeObject for Encoding {
    fn readonly_static_member(&self, name: &str) -> RuntimeResult<Val> {
        log::debug!("get_static_member called with name: {}", name);
        match name.to_ascii_lowercase().as_str() {
            //"unicode" => Ok(Val::RuntimeObject(Box::new(UnicodeEncoding {}))),
            _ => Err(RuntimeError::MemberNotFound(name.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct UnicodeEncoding {}

impl RuntimeObject for UnicodeEncoding {
    fn method(&self, name: &str) -> RuntimeResult<MethodCallType> {
        match name.to_ascii_lowercase().as_str() {
            "getstring" => Ok(Box::new(get_string)),
            _ => Err(MethodError::MethodNotFound(name.to_string()).into()),
        }
    }
}

fn get_string(_: &Val, args: Vec<Val>) -> MethodResult<Val> {
    if args.len() != 1 {
        //something wrong
        return Err(MethodError::new_incorrect_args("getstring", args));
    }

    let arg = args[0].clone();
    let Val::Array(box_vec) = arg else {
        return Err(MethodError::new_incorrect_args("getstring", args));
    };

    let v = box_vec
        .iter()
        .map(|v| {
            let Val::Char(c) = v else {
                return Err(MethodError::new_incorrect_args("getstring", args.clone()));
            };
            Ok(*c as u8)
        })
        .collect::<Result<Vec<u8>, _>>()?;

    Ok(Val::String(string_from_vec(v).into()))
}

fn string_from_vec(mut buf: Vec<u8>) -> String {
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

    res_string
}
