use std::{collections::HashMap, sync::LazyLock};

use super::{Val, ValType};
pub(crate) type JoinPredType = fn(Val, Val) -> String;

pub(crate) struct JoinPred;

impl JoinPred {
    const JOIN_PRED_MAP: LazyLock<HashMap<&'static str, JoinPredType>> =
        LazyLock::new(|| HashMap::from([("-join", join as _)]));

    pub(crate) fn get(name: &str) -> Option<JoinPredType> {
        Self::JOIN_PRED_MAP.get(name).map(|elem| *elem)
    }
}

pub fn join(input: Val, delimeter: Val) -> String {
    //strange, special case
    if input.ttype() == ValType::Null {
        if let Val::Array(box_vec) = delimeter {
            return box_vec
                .into_iter()
                .map(|v| v.cast_to_join_string())
                .collect::<Vec<String>>()
                .join("");
        } else {
            return delimeter.cast_to_string();
        }
    }

    let delimeter = delimeter.cast_to_string();
    let collection = input.cast_to_array();
    let string_vec = collection
        .into_iter()
        .map(|val| val.cast_to_string())
        .collect::<Vec<String>>();
    string_vec.join(&delimeter)
}

#[cfg(test)]
mod tests {
    use crate::PowerShellParser;

    #[test]
    fn test_join() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(r#" 1,2,3 -jOin ",,""#).unwrap(),
            "1,,2,,3".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" (1,2,3) -join ",,""#).unwrap(),
            "1,,2,,3".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" @(1,2,3) -join ",,""#).unwrap(),
            "1,,2,,3".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" 1 -join @('a', 'b', 'c') "#).unwrap(),
            "1".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -join @('a', 'b', 'c') "#).unwrap(),
            "abc".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -join @(1, 2, 3) "#).unwrap(),
            "123".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -join @('hello') "#).unwrap(),
            "hello".to_string()
        );
        assert_eq!(p.safe_eval(r#" -join @() "#).unwrap(), "".to_string());
        assert_eq!(
            p.safe_eval(r#" -join @('-join', @('a','b')) "#).unwrap(),
            "-joinSystem.Object[]".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -join @('abc', 123, $true, $null) "#)
                .unwrap(),
            "abc123True".to_string()
        );
        assert_eq!(p.safe_eval(r#" -join 'abc' "#).unwrap(), "abc".to_string());
        assert_eq!(
            p.safe_eval(r#" -join '(a,b,c)' "#).unwrap(),
            "(a,b,c)".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -join @('a', $null, 'b') "#).unwrap(),
            "ab".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" @("abc","abc") -join @('a', $null, 'b') "#)
                .unwrap(),
            "abca  babc".to_string()
        );
        assert_eq!(p.safe_eval(r#" -join (1...3) "#).unwrap(), "10".to_string());
        assert_eq!(p.safe_eval(r#" -join (1...6) "#).unwrap(), "1".to_string());
        assert_eq!(p.safe_eval(r#" -join (1..3) "#).unwrap(), "123".to_string());
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" $arr = @('x','y'); -join $arr "#)
                .unwrap(),
            "xy".to_string()
        );
    }
}
