use super::Val;

pub(crate) struct PsCommand {
    // field: Val,
    // name: String,
    // args: Vec<Val>,
}

fn normalize(input: &str, form: &str) -> Option<Val> {
    use unicode_normalization::UnicodeNormalization;

    let res = match form {
        "FormD" => input.nfd().filter(|c| c.is_ascii()).collect(), // Canonical Decomposition
        "FormC" => input.nfc().collect(),                          // Canonical Composition
        "FormKD" => input.nfkd().collect(),                        // Compatibility Decomposition
        "FormKC" => input.nfkc().collect(),                        // Compatibility Composition
        _ => input.to_string(),                                    // Default: no normalization
    };
    Some(Val::String(res))
}

impl PsCommand {
    pub fn call(field_name: Val, method_name: &str, args: Vec<Val>) -> Option<Val> {
        log::trace!(
            "PsCommand::call( {:?}, {:?}, {:?})",
            field_name,
            method_name,
            args
        );
        let method = method_name.to_ascii_lowercase();

        match method.as_str() {
            "normalize" => {
                let Val::String(field) = field_name else {
                    return None;
                };
                let Val::String(form) = args[0].clone() else {
                    return None;
                };
                normalize(field.as_str(), form.as_str())
            }
            _ => todo!(),
        }
    }
}
