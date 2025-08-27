use std::{collections::HashMap, sync::LazyLock};

use super::Val;

pub(crate) type BitwisePredType = fn(Val, Val) -> Val;

pub(crate) struct BitwisePred;

impl BitwisePred {
    const BITWISE_PRED_MAP: LazyLock<HashMap<&'static str, BitwisePredType>> =
        LazyLock::new(|| {
            HashMap::from([
                ("-band", band as _),
                ("-bor", bor as _),
                ("-bxor", bxor as _),
                ("-shl", shl as _),
                ("-shr", shr as _),
            ])
        });

    pub(crate) fn get(name: &str) -> Option<BitwisePredType> {
        Self::BITWISE_PRED_MAP
            .get(name.to_ascii_lowercase().as_str())
            .copied()
    }
}

fn band_imp(a: i64, b: i64) -> i64 {
    a & b
}

pub fn band(a: Val, b: Val) -> Val {
    let (Ok(a), Ok(b)) = (a.cast_to_int(), b.cast_to_int()) else {
        return Val::Null;
    };
    Val::Int(band_imp(a, b))
}

fn bor_imp(a: i64, b: i64) -> i64 {
    a | b
}

pub fn bor(a: Val, b: Val) -> Val {
    let (Ok(a), Ok(b)) = (a.cast_to_int(), b.cast_to_int()) else {
        return Val::Null;
    };
    Val::Int(bor_imp(a, b))
}

fn bxor_imp(a: i64, b: i64) -> i64 {
    a ^ b
}

pub fn bxor(a: Val, b: Val) -> Val {
    let (Ok(a), Ok(b)) = (a.cast_to_int(), b.cast_to_int()) else {
        return Val::Null;
    };
    Val::Int(bxor_imp(a, b))
}

fn shl_imp(a: i64, b: i64) -> i64 {
    a << b
}

pub fn shl(a: Val, b: Val) -> Val {
    let (Ok(a), Ok(b)) = (a.cast_to_int(), b.cast_to_int()) else {
        return Val::Null;
    };
    Val::Int(shl_imp(a, b))
}

fn shr_imp(a: i64, b: i64) -> i64 {
    a >> b
}

pub fn shr(a: Val, b: Val) -> Val {
    let (Ok(a), Ok(b)) = (a.cast_to_int(), b.cast_to_int()) else {
        return Val::Null;
    };
    Val::Int(shr_imp(a, b))
}

#[cfg(test)]
mod tests {
    use crate::PowerShellSession;

    #[test]
    fn test_band() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" 5 -band 4 "#).unwrap(), "4".to_string());
        assert_eq!(p.safe_eval(r#" 5 -band 2 "#).unwrap(), "0".to_string());
        assert_eq!(p.safe_eval(r#" 5 -Band 9 "#).unwrap(), "1".to_string());
    }

    #[test]
    fn test_bor() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" 5 -bOr 4 "#).unwrap(), "5".to_string());
        assert_eq!(p.safe_eval(r#" 5 -bor 2 "#).unwrap(), "7".to_string());
        assert_eq!(p.safe_eval(r#" 5 -bor 9 "#).unwrap(), "13".to_string());
        assert_eq!(
            p.safe_eval(r#" 5 -bor 5 -band 4 "#).unwrap(),
            "4".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" 5 -band 5 -bor 4 "#).unwrap(),
            "5".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" 6 -bor 5 -bor 4 "#).unwrap(),
            "7".to_string()
        );
    }

    #[test]
    fn test_bxor() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" 5 -bxor 4 "#).unwrap(), "1".to_string());
        assert_eq!(p.safe_eval(r#" 5 -bxor 2 "#).unwrap(), "7".to_string());
        assert_eq!(p.safe_eval(r#" 5 -bxor 9 "#).unwrap(), "12".to_string());
    }

    #[test]
    fn test_shl() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" 5 -shl 4 "#).unwrap(), "80".to_string());
        assert_eq!(p.safe_eval(r#" -5 -shl 2 "#).unwrap(), "-20".to_string());
        assert_eq!(
            p.safe_eval(r#" "5.5" -shl 3.5 "#).unwrap(),
            "96".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "+5.5" -sHl 3.5 "#).unwrap(),
            "96".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "5.5as" -shl 3.5 "#).unwrap(),
            "".to_string()
        );
    }

    #[test]
    fn test_shr() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" 96 -shr 4 "#).unwrap(), "6".to_string());
        assert_eq!(p.safe_eval(r#" -96 -shr 2 "#).unwrap(), "-24".to_string());
        assert_eq!(
            p.safe_eval(r#" "96.5" -shr 3.5 "#).unwrap(),
            "6".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "+96.5" -shr 3.5 "#).unwrap(),
            "6".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "96.5as" -shr 3.5 "#).unwrap(),
            "".to_string()
        );
    }

    #[test]
    fn test_bnot() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" -bnot 4 "#).unwrap(), "-5".to_string());
        assert_eq!(p.safe_eval(r#" -bnot -95 "#).unwrap(), "94".to_string());
        assert_eq!(p.safe_eval(r#" [int] "96.5" "#).unwrap(), "96".to_string());
        assert_eq!(p.safe_eval(r#" [int] "97.5" "#).unwrap(), "98".to_string());
        assert_eq!(p.safe_eval(r#" -bnot "96.5" "#).unwrap(), "-97".to_string());
        assert_eq!(p.safe_eval(r#" -bnot "97" "#).unwrap(), "-98".to_string());
        assert_eq!(
            p.safe_eval(r#" -bnot "+96.5" "#).unwrap(),
            "-97".to_string()
        );
        assert_eq!(p.safe_eval(r#" -bnot "96.5as" "#).unwrap(), "".to_string());
        assert_eq!(
            p.safe_eval(r#" [float]"+96.51e1" "#).unwrap(),
            "965.1".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" [int]"+96.51e1" "#).unwrap(),
            "965".to_string()
        );
        assert_eq!(p.safe_eval(r#" [int]"+96e1" "#).unwrap(), "960".to_string());
        assert_eq!(p.safe_eval(r#" [int]"0x96" "#).unwrap(), "150".to_string());
        assert_eq!(
            p.safe_eval(r#" [int]"0x96e1" "#).unwrap(),
            "38625".to_string()
        );
        assert_eq!(p.safe_eval(r#" [int]"+0x96e1" "#).unwrap(), "".to_string());
        assert_eq!(
            p.safe_eval(r#" -bnot "+96.51e1" "#).unwrap(),
            "-966".to_string()
        );
    }
}
