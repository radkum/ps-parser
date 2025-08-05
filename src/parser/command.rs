use super::Val;
use thiserror_no_std::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CommandError {
    #[error("Method \"{0}\" not found")]
    MethodNotFound(String),

    #[error("Object \"{0}\" not found")]
    ObjectNotFound(String),

    #[error("Incorrect arguments \"{1:?}\" for method \"{0}\"")]
    IncorrectArgs(String, Vec<String>),

}
type CommandResult<T> = core::result::Result<T, CommandError>;

pub(crate) struct PsCommand {
    // field: Val,
    // name: String,
    // args: Vec<Val>,
}

fn normalize(input: &str, form: &str) -> Val {
    use unicode_normalization::UnicodeNormalization;

    let res = match form {
        "FormD" => input.nfd().filter(|c| c.is_ascii()).collect(), // Canonical Decomposition
        "FormC" => input.nfc().collect(),                          // Canonical Composition
        "FormKD" => input.nfkd().collect(),                        // Compatibility Decomposition
        "FormKC" => input.nfkc().collect(),                        // Compatibility Composition
        _ => input.to_string(),                                    // Default: no normalization
    };
    Val::String(res)
}

impl PsCommand {
    pub fn call(field_name: Val, method_name: &str, args: Vec<Val>) -> CommandResult<Val> {
        log::trace!(
            "PsCommand::call( {:?}, {:?}, {:?})",
            field_name,
            method_name,
            args
        );
        let Val::String(field) = field_name else {
            return Err(CommandError::ObjectNotFound(field_name.cast_to_string()));
        };

        let method = method_name.to_ascii_lowercase();
        match method.as_str() {
            "normalize" => {
                let Val::String(form) = args[0].clone() else {
                    return Err(CommandError::IncorrectArgs(method, args.into_iter().map(|v|v.cast_to_string()).collect::<Vec<String>>()));
                };
                Ok(normalize(field.as_str(), form.as_str()))
            }
            _ => return Err(CommandError::MethodNotFound(method)),
        }
    }
}
