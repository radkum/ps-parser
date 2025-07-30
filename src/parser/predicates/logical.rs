use std::{collections::HashMap, sync::LazyLock};

use super::Val;

pub(crate) type LogicalPredType = fn(Val, Val) -> bool;

pub(crate) struct LogicalPred;

impl LogicalPred {
    const LOGICAL_PRED_MAP: LazyLock<HashMap<&'static str, LogicalPredType>> =
        LazyLock::new(|| HashMap::from([("-and", and as _), ("-or", or as _), ("-xor", xor as _)]));

    pub(crate) fn get(name: &str) -> Option<LogicalPredType> {
        Self::LOGICAL_PRED_MAP.get(name).map(|elem| *elem)
    }
}

pub fn and(a: Val, b: Val) -> bool {
    a.cast_to_bool() && b.cast_to_bool()
}

pub fn or(a: Val, b: Val) -> bool {
    a.cast_to_bool() || b.cast_to_bool()
}

pub fn xor(a: Val, b: Val) -> bool {
    a.cast_to_bool() != b.cast_to_bool()
}

#[cfg(test)]
mod tests {
    use crate::{PowerShellParser, parser::ParserError};

    #[test]
    fn test_and() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(r#" $true -and $true "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $true -and $false "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $false -and $true "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $false -and $false "#).unwrap(),
            "False".to_string()
        );
    }

    #[test]
    fn test_or() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(r#" $true -or $true "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $true -or $false "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $false -or $true "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $false -or $false "#).unwrap(),
            "False".to_string()
        );

        assert!(matches!(
            p.safe_eval(r#" $false -or $false -or $false "#)
                .unwrap_err(),
            ParserError::PestError(_)
        ));
    }

    #[test]
    fn test_xor() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(r#" $true -xor $true "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $true -xor $false "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $false -xor $true "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $false -xor $false "#).unwrap(),
            "False".to_string()
        );
    }

    #[test]
    fn test_not() {
        let mut p = PowerShellParser::new();
        assert_eq!(p.safe_eval(r#" -not 4 "#).unwrap(), "False".to_string());
        assert_eq!(p.safe_eval(r#" -not "" "#).unwrap(), "True".to_string());
        assert_eq!(p.safe_eval(r#" -not "asd" "#).unwrap(), "False".to_string());
        assert_eq!(
            p.safe_eval(r#" -not "96.5" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -not "+96.5" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -not "96.5as" "#).unwrap(),
            "False".to_string()
        );
    }
}
