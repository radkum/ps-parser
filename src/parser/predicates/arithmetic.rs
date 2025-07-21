use std::{collections::HashMap, sync::LazyLock};

use super::Val;

fn add(mut a: Val, b: Val) -> Val {
    if let Err(err) = a.add(b) {
        log::warn!("{err}");
        Val::Null
    } else {
        a
    }
}

fn sub(mut a: Val, b: Val) -> Val {
    if let Err(err) = a.sub(b) {
        log::warn!("{err}");
        Val::Null
    } else {
        a
    }
}

fn mul(mut a: Val, b: Val) -> Val {
    if let Err(err) = a.mul(b) {
        log::warn!("{err}");
        Val::Null
    } else {
        a
    }
}

fn div(mut a: Val, b: Val) -> Val {
    if let Err(err) = a.div(b) {
        log::warn!("{err}");
        Val::Null
    } else {
        a
    }
}

fn modulo(mut a: Val, b: Val) -> Val {
    if let Err(err) = a.modulo(b) {
        log::warn!("{err}");
        Val::Null
    } else {
        a
    }
}

fn assign(_arg1: Val, arg2: Val) -> Val {
    arg2
}

pub(crate) type PredType = fn(Val, Val) -> Val;

pub(crate) struct ArithmeticPred;

impl ArithmeticPred {
    const ARYTHMETIC_PRED_MAP: LazyLock<HashMap<&'static str, PredType>> = LazyLock::new(|| {
        HashMap::from([
            ("+", add as _),
            ("-", sub as _),
            ("*", mul as _),
            ("/", div as _),
            ("%", modulo as _),
            ("=", assign as _),
        ])
    });

    pub(crate) fn get(name: &str) -> Option<PredType> {
        Self::ARYTHMETIC_PRED_MAP.get(name).map(|elem| *elem)
    }
}

#[cfg(test)]
mod tests {
    use crate::PowerShellParser;

    #[test]
    fn test_add() {
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 0123  $true " + 0.1 "#)
                .unwrap(),
            " 0123  True 0.1".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" 1 + " 1" + "4  " + $asdf "#)
                .unwrap(),
            "6".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#"$asdf += 1 + " 1" + "4  " + $asdf; $asdf"#)
                .unwrap(),
            "6".to_string()
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 0123 " - 0.1 "#)
                .unwrap(),
            "122.9".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 0123  $true " - 0.1 "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" 1 - " 1" + "4  " - $asdf "#)
                .unwrap(),
            "4".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#"$asdf -= 1 + " 1" - "4  " + $asdf; $asdf"#)
                .unwrap(),
            "2".to_string()
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" 8*8 "#).unwrap(),
            "64".to_string()
        );
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" 8*" 7 " "#).unwrap(),
            "56".to_string()
        );
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" " 8 "* 2 "#).unwrap(),
            " 8  8 ".to_string()
        );
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" " 8a "* 2 "#).unwrap(),
            " 8a  8a ".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 8a "* " 2" "#)
                .unwrap(),
            " 8a  8a ".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 8a "* " 2a" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#"$asdf = 1 + " 1" - "4  " + $asdf; $asdf*5"#)
                .unwrap(),
            "-10".to_string()
        );
    }

    #[test]
    fn test_div() {
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" 8/8 "#).unwrap(),
            "1".to_string()
        );
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" 8/" 16 " "#).unwrap(),
            "0.5".to_string()
        );
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" " 8 "/ 2 "#).unwrap(),
            "4".to_string()
        );
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" " 8a "/ 2 "#).unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 8a "/ " 2" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 8 "/ " 2a" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#"$asdf = 1 + " 1" - "4  " + $asdf; $asdf/=5;$asdf"#)
                .unwrap(),
            "-0.4".to_string()
        );
    }

    #[test]
    fn test_mod() {
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" 8%8 "#).unwrap(),
            "0".to_string()
        );
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" 8%7 "#).unwrap(),
            "1".to_string()
        );
        assert_eq!(
            PowerShellParser::new().safe_eval(r#" 8%" 16 " "#).unwrap(),
            "8".to_string()
        );
        //assert_eq!(PowerShellParser::new().safe_eval(r#" " 8 "% 0.3
        // "#).unwrap(), "0.2".to_string());
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 8 "% 0.3 "#)
                .unwrap(),
            "0.2000000000000003".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 8a "% 0.2 "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 8a "% " 2" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" " 8 "% " 2a" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#"$asdf = 1 + " 1" % "4  " + $asdf; $asdf%=5;$asdf"#)
                .unwrap(),
            "2".to_string()
        );
    }

    #[test]
    fn test_cast() {
        let mut p = PowerShellParser::new();
        assert_eq!(p.safe_eval("[lonG](97 + 3)").unwrap(), "100".to_string());
        assert_eq!(
            p.safe_eval("[doUble](97 + 3.1)").unwrap(),
            "100.1".to_string()
        );
        assert_eq!(p.safe_eval("[char](97 + 1)").unwrap(), "b".to_string());
        assert_eq!(
            p.safe_eval("[bYte][char](97 + 1)").unwrap(),
            "b".to_string()
        );
        assert_eq!(p.safe_eval("[bool]0.09874").unwrap(), "True".to_string());
        assert_eq!(p.safe_eval(r#" [BOOl]"" "#).unwrap(), "False".to_string());
    }
}
