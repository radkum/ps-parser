use super::{MethodError, MethodResult, PsString, RuntimeObject, StaticFnCallType, Val};
use crate::parser::value::runtime_object::RuntimeResult;

#[derive(Debug, Clone)]
pub(crate) struct Convert {}

impl RuntimeObject for Convert {
    fn get_static_fn(&self, name: &str) -> RuntimeResult<StaticFnCallType> {
        match name.to_ascii_lowercase().as_str() {
            "frombase64string" => Ok(from_base_64_string),
            _ => Err(MethodError::MethodNotFound(name.to_string()).into()),
        }
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
