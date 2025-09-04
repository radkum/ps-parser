use std::{collections::HashMap, sync::LazyLock};

use super::{Val, ValResult};

fn add(mut a: Val, b: Val) -> ValResult<Val> {
    a.add(b)?;
    Ok(a)
}

fn sub(mut a: Val, b: Val) -> ValResult<Val> {
    a.sub(b)?;
    Ok(a)
}

fn mul(mut a: Val, b: Val) -> ValResult<Val> {
    a.mul(b)?;
    Ok(a)
}

fn div(mut a: Val, b: Val) -> ValResult<Val> {
    a.div(b)?;
    Ok(a)
}

fn modulo(mut a: Val, b: Val) -> ValResult<Val> {
    a.modulo(b)?;
    Ok(a)
}

fn assign(_arg1: Val, arg2: Val) -> ValResult<Val> {
    Ok(arg2)
}

pub(crate) type PredType = fn(Val, Val) -> ValResult<Val>;

pub(crate) struct ArithmeticPred;

impl ArithmeticPred {
    const ARYTHMETIC_PRED_MAP: LazyLock<HashMap<&'static str, PredType>> = LazyLock::new(|| {
        HashMap::from([
            ("+", add as PredType),
            ("-", sub as PredType),
            ("*", mul as PredType),
            ("/", div as PredType),
            ("%", modulo as PredType),
            ("=", assign as PredType),
        ])
    });

    pub(crate) fn get(name: &str) -> Option<PredType> {
        Self::ARYTHMETIC_PRED_MAP.get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::{NEWLINE, PowerShellSession, Variables};

    #[test]
    fn test_add() {
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 0123  $true " + 0.1 "#)
                .unwrap(),
            " 0123  True 0.1".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .with_variables(Variables::force_eval())
                .safe_eval(r#" 1 + " 1" + "4  " + $asdf "#)
                .unwrap(),
            "6".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .with_variables(Variables::force_eval())
                .safe_eval(r#"$asdf += 1 + " 1" + "4  " + $asdf; $asdf"#)
                .unwrap(),
            "6".to_string()
        );
    }

    #[test]
    fn test_sub() {
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 0123 " - 0.1 "#)
                .unwrap(),
            "122.9".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 0123  $true " - 0.1 "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .with_variables(Variables::force_eval())
                .safe_eval(r#" 1 - " 1" + "4  " - $asdf "#)
                .unwrap(),
            "4".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .with_variables(Variables::force_eval())
                .safe_eval(r#"$asdf -= 1 + " 1" - "4  " + $asdf; $asdf"#)
                .unwrap(),
            "2".to_string()
        );
    }

    #[test]
    fn test_mul() {
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" 8*8 "#).unwrap(),
            "64".to_string()
        );
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" 8*" 7 " "#).unwrap(),
            "56".to_string()
        );
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" " 8 "* 2 "#).unwrap(),
            " 8  8 ".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8a "* 2 "#)
                .unwrap(),
            " 8a  8a ".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8a "* " 2" "#)
                .unwrap(),
            " 8a  8a ".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8a "* " 2a" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .with_variables(Variables::force_eval())
                .safe_eval(r#"$asdf = 1 + " 1" - "4  " + $asdf; $asdf*5"#)
                .unwrap(),
            "-10".to_string()
        );
    }

    #[test]
    fn test_div() {
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" 8/8 "#).unwrap(),
            "1".to_string()
        );
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" 8/" 16 " "#).unwrap(),
            "0.5".to_string()
        );
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" " 8 "/ 2 "#).unwrap(),
            "4".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8a "/ 2 "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8a "/ " 2" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8 "/ " 2a" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .with_variables(Variables::force_eval())
                .safe_eval(r#"$asdf = 1 + " 1" - "4  " + $asdf; $asdf/=5;$asdf"#)
                .unwrap(),
            "-0.4".to_string()
        );
    }

    #[test]
    fn test_mod() {
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" 8%8 "#).unwrap(),
            "0".to_string()
        );
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" 8%7 "#).unwrap(),
            "1".to_string()
        );
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" 8%" 16 " "#).unwrap(),
            "8".to_string()
        );
        //assert_eq!(PowerShellParser::new().safe_eval(r#" " 8 "% 0.3
        // "#).unwrap(), "0.2".to_string());
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8 "% 0.3 "#)
                .unwrap(),
            "0.2000000000000003".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8a "% 0.2 "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8a "% " 2" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" " 8 "% " 2a" "#)
                .unwrap(),
            "".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .with_variables(Variables::force_eval())
                .safe_eval(r#"$asdf = 1 + " 1" % "4  " + $asdf; $asdf%=5;$asdf"#)
                .unwrap(),
            "2".to_string()
        );
    }

    #[test]
    fn test_cast() {
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [INt](70+44-44) "#)
                .unwrap()
                .as_str(),
            "70"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval("[lonG](97 + 3)")
                .unwrap(),
            "100".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval("[doUble](97 + 3.1)")
                .unwrap(),
            "100.1".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval("[char](97 + 1)")
                .unwrap(),
            "b".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval("[bYte][char](97 + 1)")
                .unwrap(),
            "b".to_string()
        );
        assert_eq!(
            PowerShellSession::new().safe_eval("[bool]0.09874").unwrap(),
            "True".to_string()
        );
        assert_eq!(
            PowerShellSession::new().safe_eval(r#" [BOOl]"" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [Bool] @(1,2.3, "asdf", $null, $true) "#)
                .unwrap()
                .as_str(),
            "True"
        );

        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [chaR](70+44-44) "#)
                .unwrap()
                .as_str(),
            "F"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [string](70+44-44) "#)
                .unwrap()
                .as_str(),
            "70"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [string]$true "#)
                .unwrap()
                .as_str(),
            "True"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [string][int]$true "#)
                .unwrap()
                .as_str(),
            "1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [string] "asdfad" "#)
                .unwrap()
                .as_str(),
            "asdfad"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [string] .0 "#)
                .unwrap()
                .as_str(),
            "0"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [string] @(1,2.3, "asdf", $null, $true) "#)
                .unwrap()
                .as_str(),
            "1 2.3 asdf  True"
        );

        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [arraY](70+44-44) "#)
                .unwrap()
                .as_str(),
            "70"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [arraY]$true "#)
                .unwrap()
                .as_str(),
            "True"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [arraY][int]$true "#)
                .unwrap()
                .as_str(),
            "1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [arraY] "asdfad" "#)
                .unwrap()
                .as_str(),
            "asdfad"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [arraY] .0 "#)
                .unwrap()
                .as_str(),
            "0"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [arraY] @(1,2.3) "#)
                .unwrap()
                .as_str(),
            vec!["1", "2.3"].join(NEWLINE)
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" [arraY] (1,2.3) "#)
                .unwrap()
                .as_str(),
            vec!["1", "2.3"].join(NEWLINE)
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" (1,2.3) "#)
                .unwrap()
                .as_str(),
            vec!["1", "2.3"].join(NEWLINE)
        );
    }

    #[test]
    fn test_pre_inc() {
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" ++($a);$a "#)
                .unwrap()
                .as_str(),
            "1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = 70;++($a);$a "#)
                .unwrap()
                .as_str(),
            "71"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = 70;$b=++$a;$b "#)
                .unwrap()
                .as_str(),
            "71"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [Float](70+44-44)+0.1;++$a;$a "#)
                .unwrap()
                .as_str(),
            "71.1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [StRing](70+44-44);++$a;$a "#)
                .unwrap()
                .as_str(),
            "70"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [cHar](70+44-44);++$a;$a "#)
                .unwrap()
                .as_str(),
            "F"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [bool](70+44-44);++$a;$a "#)
                .unwrap()
                .as_str(),
            "True"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [array](70+44-44);++$a;$a "#)
                .unwrap()
                .as_str(),
            "70"
        );
    }

    #[test]
    fn test_pre_dec() {
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" --($a);$a "#)
                .unwrap()
                .as_str(),
            "-1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = 70;--($a);$a "#)
                .unwrap()
                .as_str(),
            "69"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = 70;--$a;$a "#)
                .unwrap()
                .as_str(),
            "69"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [Float](70+44-44)+0.1;--$a;$a "#)
                .unwrap()
                .as_str(),
            "69.1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [StRing](70+44-44);--$a;$a "#)
                .unwrap()
                .as_str(),
            "70"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [cHar](70+44-44);--$a;$a "#)
                .unwrap()
                .as_str(),
            "F"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [bool](70+44-44);--$a;$a "#)
                .unwrap()
                .as_str(),
            "True"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [array](70+44-44);--$a;$a "#)
                .unwrap()
                .as_str(),
            "70"
        );
    }

    #[test]
    fn test_post_inc() {
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" ($a)++;$a "#)
                .unwrap()
                .as_str(),
            "1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = 70;($a)++;$a "#)
                .unwrap()
                .as_str(),
            "71"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = 70;$b=$a++;$b "#)
                .unwrap()
                .as_str(),
            "70"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [Float](70+44-44)+0.1;$b=$a++;$b "#)
                .unwrap()
                .as_str(),
            "70.1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [StRing](70+44-44);$b=$a++;$b "#)
                .unwrap()
                .as_str(),
            ""
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [cHar](70+44-44);$b=$a++;$b "#)
                .unwrap()
                .as_str(),
            ""
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [bool](70+44-44);$b=$a++;$b "#)
                .unwrap()
                .as_str(),
            ""
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [array](70+44-44);$b=$a++;$b "#)
                .unwrap()
                .as_str(),
            ""
        );
    }

    #[test]
    fn test_post_dec() {
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" ($a)--;$a "#)
                .unwrap()
                .as_str(),
            "-1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" (($a))--;$a "#)
                .unwrap()
                .as_str(),
            "-1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = 70;($a)--;$a "#)
                .unwrap()
                .as_str(),
            "69"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = 70;$b=$a--;$b "#)
                .unwrap()
                .as_str(),
            "70"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [Float](70+44-44)+0.1;$b=$a--;$b "#)
                .unwrap()
                .as_str(),
            "70.1"
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [StRing](70+44-44);$b=$a--;$b "#)
                .unwrap()
                .as_str(),
            ""
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [cHar](70+44-44);$b=$a--;$b "#)
                .unwrap()
                .as_str(),
            ""
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [bool](70+44-44);$b=$a--;$b "#)
                .unwrap()
                .as_str(),
            ""
        );
        assert_eq!(
            PowerShellSession::new()
                .safe_eval(r#" $a = [array](70+44-44);$b=$a--;$b "#)
                .unwrap()
                .as_str(),
            ""
        );
    }
}
