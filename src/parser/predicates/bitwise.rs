use std::{collections::HashMap, sync::LazyLock};

use thiserror_no_std::Error;

use super::Val;
use crate::parser::value::ValError;
#[derive(Error, Debug, PartialEq, Clone)]
pub enum BitwiseError {
    #[error("{0} not defined for {1}")]
    NotDefined(String, String),
    #[error("Failed casting to int: {0}")]
    CastToInt(ValError),
}

impl From<ValError> for BitwiseError {
    fn from(value: ValError) -> Self {
        Self::CastToInt(value)
    }
}

type BitwiseResult<T> = core::result::Result<T, BitwiseError>;

pub(crate) type BitwisePredType = fn(Val, Val) -> BitwiseResult<Val>;

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

pub fn prepare(a: Val, b: Val, op_name: &str) -> BitwiseResult<(i64, i64)> {
    if let Val::RuntimeObject(ro) = a {
        return Err(BitwiseError::NotDefined(op_name.into(), ro.name()));
    }
    if let Val::RuntimeType(rt) = a {
        return Err(BitwiseError::NotDefined(op_name.into(), rt.name()));
    }
    Ok((a.cast_to_int()?, b.cast_to_int()?))
}

fn band_imp(a: i64, b: i64) -> i64 {
    a & b
}

pub fn band(a: Val, b: Val) -> BitwiseResult<Val> {
    let (a, b) = prepare(a, b, "-band")?;
    let res = band_imp(a, b);
    Ok(Val::Int(res))
}

fn bor_imp(a: i64, b: i64) -> i64 {
    a | b
}

pub fn bor(a: Val, b: Val) -> BitwiseResult<Val> {
    let (a, b) = prepare(a, b, "-bor")?;
    let res = bor_imp(a, b);
    Ok(Val::Int(res))
}

fn bxor_imp(a: i64, b: i64) -> i64 {
    a ^ b
}

pub fn bxor(a: Val, b: Val) -> BitwiseResult<Val> {
    let (a, b) = prepare(a, b, "-bxor")?;
    let res = bxor_imp(a, b);
    Ok(Val::Int(res))
}

fn shl_imp(a: i64, b: i64) -> i64 {
    a << b
}

pub fn shl(a: Val, b: Val) -> BitwiseResult<Val> {
    let (a, b) = prepare(a, b, "-shl")?;
    let res = shl_imp(a, b);
    Ok(Val::Int(res))
}

fn shr_imp(a: i64, b: i64) -> i64 {
    a >> b
}

pub fn shr(a: Val, b: Val) -> BitwiseResult<Val> {
    let (a, b) = prepare(a, b, "-shr")?;
    let res = shr_imp(a, b);
    Ok(Val::Int(res))
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
