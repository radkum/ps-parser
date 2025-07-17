use std::{collections::HashMap, sync::LazyLock};

use super::Val;
use crate::parser::ValType;

pub(crate) type TypeCheckPredType = fn(Val, ValType) -> bool;

pub(crate) struct TypeCheckPred;

impl TypeCheckPred {
    const TYPECHECK_PRED_MAP: LazyLock<HashMap<&'static str, TypeCheckPredType>> =
        LazyLock::new(|| HashMap::from([("-is", is as _), ("-isnot", isnot as _)]));

    pub(crate) fn get(name: &str) -> Option<TypeCheckPredType> {
        Self::TYPECHECK_PRED_MAP.get(name).map(|elem| *elem)
    }
}

pub fn is(var: Val, ttype: ValType) -> bool {
    var.ttype() == ttype
}

fn isnot(var: Val, ttype: ValType) -> bool {
    !is(var, ttype)
}

#[cfg(test)]
mod tests {
    use crate::{
        PowerShellParser,
        parser::{ParserError::ValError, ValError::UnknownType},
    };
    #[test]
    fn test_typecheck() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.evaluate_last_exp(r#" 42 -isnot [string] "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.evaluate_last_exp(r#" 42 -isnot [asdfas] "#).unwrap_err(),
            ValError(UnknownType("asdfas".to_string()))
        );
        assert_eq!(
            p.evaluate_last_exp(r#" 42 -is [int] "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.evaluate_last_exp(r#" 42 -is [inT] "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.evaluate_last_exp(r#" [chAr]42 -is [ChaR] "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.evaluate_last_exp(r#" [chAr]42 -is [string] "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.evaluate_last_exp(r#" ([chAr]42+[char]33) -is [string] "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.evaluate_last_exp(r#" 55. -is [float] "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.evaluate_last_exp(r#" 42 -is [float] "#).unwrap(),
            "False".to_string()
        );
    }
}
