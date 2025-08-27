mod arithmetic;
mod bitwise;
mod comparison;
mod contain;
mod join;
mod logical;
mod replace;
mod split;
mod type_check;

pub(crate) use arithmetic::ArithmeticPred;
pub(crate) use bitwise::BitwisePred;
pub(crate) use comparison::ComparisonPred;
pub(crate) use contain::ContainPred;
pub(crate) use join::JoinPred;
pub(crate) use logical::LogicalPred;
pub(crate) use replace::ReplacePred;
pub(crate) use split::SplitPred;
use thiserror_no_std::Error;
pub(crate) use type_check::TypeCheckPred;

use super::{Val, ValResult, ValType};

#[derive(Error, Debug, PartialEq, Clone)]
pub enum OpError {
    #[error("The -ireplace operator allows only two elements to follow it, not {0}")]
    ReplaceInvalidArgsNumber(usize),
}

type OpResult<T> = core::result::Result<T, OpError>;

pub(crate) type StringPredType = Box<dyn Fn(Val, Val) -> OpResult<Val>>;

pub(crate) struct StringPred;
impl StringPred {
    pub(crate) fn get(name: &str) -> Option<StringPredType> {
        let name_lowercase = name.to_ascii_lowercase();
        if let Some(compare) = ComparisonPred::get(name_lowercase.as_str()) {
            return Some(Box::new(move |v1, v2| Ok(Val::Bool(compare(v1, v2)))));
        }

        if let Some(replace) = ReplacePred::get(name_lowercase.as_str()) {
            return Some(Box::new(move |v1, v2| {
                let (from, to) = if let Val::Array(arr) = v2 {
                    if arr.len() == 1 {
                        (arr[0].clone(), Val::Null)
                    } else if arr.len() == 2 {
                        (arr[0].clone(), arr[1].clone())
                    } else {
                        Err(OpError::ReplaceInvalidArgsNumber(arr.len()))?
                    }
                } else {
                    (v2, Val::Null)
                };
                Ok(Val::String(replace(v1, from, to).into()))
            }));
        }

        if let Some(type_check) = TypeCheckPred::get(name_lowercase.as_str()) {
            return Some(Box::new(move |v1, v2| {
                Ok(Val::Bool(type_check(v1, v2.ttype())))
            }));
        }

        if let Some(join) = JoinPred::get(name_lowercase.as_str()) {
            return Some(Box::new(move |v1, v2| Ok(Val::String(join(v1, v2).into()))));
        }

        if let Some(split) = SplitPred::get(name_lowercase.as_str()) {
            return Some(Box::new(move |v1, v2| Ok(split(v1, v2))));
        }

        if let Some(contain) = ContainPred::get(name_lowercase.as_str()) {
            return Some(Box::new(move |v1, v2| Ok(Val::Bool(contain(v1, v2)))));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::PowerShellSession;

    #[test]
    fn test_obfuscation() {
        let mut p = PowerShellSession::new();

        assert_eq!(p.safe_eval(r#" [CHAR](70+44-44)+[chaR](81+30)+[ChAR]([byTE]0x72)+[CHar]([byTE]0x6d)+[CHAR](68) "#).unwrap().as_str(), "FormD");
        assert_eq!(p.safe_eval(r#" [Char]([BYte]0x5c)+[ChAr](112)+[chAR]([bYTE]0x7b)+[ChAr]([BYtE]0x4d)+[Char](110)+[CHar]([bYte]0x7d) "#).unwrap().as_str(), "\\p{Mn}");
        assert_eq!(
            p.safe_eval(r#" $(('WrìtêÍnt32').NoRMaLIZE("FormD") -replace "\p{Mn}") "#)
                .unwrap()
                .as_str(),
            "WriteInt32"
        );
        assert_eq!(
            p.safe_eval(r#" ("$(('WrìtêÍnt32').NoRMaLIZE("FormD") -replace "\p{Mn}")") "#)
                .unwrap()
                .as_str(),
            "WriteInt32"
        );
        assert_eq!(p.safe_eval(r#" ("$(('Wrì'+'têÍ'+'nt3'+'2').NoRMaLIZE([CHAR](70+44-44)+[chaR](81+30)+[ChAR]([byTE]0x72)+[CHar]([byTE]0x6d)+[CHAR](68)) -replace [Char]([BYte]0x5c)+[ChAr](112)+[chAR]([bYTE]0x7b)+[ChAr]([BYtE]0x4d)+[Char](110)+[CHar]([bYte]0x7d))") "#).unwrap().as_str(), "WriteInt32");
        assert_eq!(p.safe_eval(r#" $(('àmsìCónté'+'xt').normaliZE([Char]([byTe]0x46)+[cHAR]([BYTe]0x6f)+[cHaR](11+103)+[chAr](109*21/21)+[Char](68)) -replace [CHaR](92+6-6)+[CHAR](112*58/58)+[cHar]([bYte]0x7b)+[cHar](77*36/36)+[cHAR](110)+[char](125)) "#).unwrap().as_str(), "amsiContext");
        assert_eq!(p.safe_eval(r#" $ykHjp2N3fNRJs="System.$([CHAr]([BYTe]0x4d)+[char](97*89/89)+[chAr](110*12/12)+[Char]([byTe]0x61)+[CHAR]([bYtE]0x67)+[ChAr]([BYTe]0x65)+[CHAr](109+54-54)+[cHAR]([ByTE]0x65)+[ChAR]([BYtE]0x6e)+[CHaR](45+71)).$([ChaR]([ByTE]0x41)+[cHAR](12+105)+[cHar]([ByTe]0x74)+[CHar]([byTE]0x6f)+[CHaR](109)+[ChAR]([byTe]0x61)+[cHaR]([byTE]0x74)+[cHAR](105*19/19)+[CHaR](111+20-20)+[cHAR]([BYTE]0x6e)).$(('ÂmsìÛ'+'tíls').NOrmaLiZE([ChAR]([byTE]0x46)+[cHaR](111+19-19)+[cHaR](114+36-36)+[char](109)+[chAr]([BytE]0x44)) -replace [CHar](92+37-37)+[ChAR](112)+[cHAR](123+45-45)+[cHAR](77*55/55)+[chAR]([BYtE]0x6e)+[CHar]([bYTe]0x7d))"; $ykHjp2N3fNRJs "#).unwrap().as_str(), "System.Management.Automation.AmsiUtils");
    }

    #[test]
    fn test_range_with_float() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" [string](1..1.3) "#).unwrap().as_str(), "1");
        assert_eq!(p.safe_eval(r#" [string](1...3) "#).unwrap().as_str(), "1 0");
        assert_eq!(p.safe_eval(r#" [string]1...3 "#).unwrap().as_str(), "1 0");
    }

    #[test]
    fn test_unary() {
        let mut p = PowerShellSession::new();
        assert_eq!(p.safe_eval(r#" +5 "#).unwrap().as_str(), "5");
        assert_eq!(p.safe_eval(r#" -5 "#).unwrap().as_str(), "-5");
    }

    #[test]
    fn test_format_operator() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(r#" "Hello, {0}!" -f "world" "#)
                .unwrap()
                .as_str(),
            "Hello, world!"
        );
        assert_eq!(
            p.safe_eval(r#" "Hello, {0}!" -f "every{0}" -f "body"  "#)
                .unwrap()
                .as_str(),
            "Hello, everybody!"
        );
        assert_eq!(
            p.safe_eval(r#" "{0} + {1} = {2}" -f 5, 7, (5 + 7) "#)
                .unwrap()
                .as_str(),
            "5 + 7 = 12"
        );
        assert_eq!(
            p.safe_eval(r#" "{0:N2}" -f 1234.56789 "#).unwrap().as_str(),
            "1234.57"
        );
        assert_eq!(
            p.safe_eval(r#" "|{0,10}|" -f "Hi" "#).unwrap().as_str(),
            "|          Hi|"
        );
        assert_eq!(
            p.safe_eval(
                r#" $level = "INFO";$message = "Disk space low";"{0}: {1}" -f $level, $message "#
            )
            .unwrap()
            .as_str(),
            "INFO: Disk space low"
        );
        assert_eq!(
            p.safe_eval(r#" "{0:310100a0b00}" -f 578 "#)
                .unwrap()
                .as_str(),
            "310100a5b78"
        );

        //veeeery strange cases
        //assert_eq!(p.safe_eval(r#"
        // "{0:31sdfg,0100a0b00000000000000000000000}" -f
        // 57899999999999999999999999999 "#).unwrap().as_str(),
        // "31sdfg578199a9b99999999999999999999999"); assert_eq!(p.
        // safe_eval(r#" "{0:31sdfg,0100a0b00000000000000000000000}" -f
        // 578999999999999999999999999999 "#).unwrap().as_str(),
        // "31sdfg5790100a0b00000000000000000000000");
    }

    #[test]
    fn test_strings() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(r#" 'It''s fine' "#).unwrap().as_str(),
            "It''s fine"
        );
        assert_eq!(
            p.safe_eval(r#" "Price is $" "#).unwrap().as_str(),
            "Price is $"
        );
        assert_eq!(
            p.safe_eval(r#" "Result: $(1+2)" "#).unwrap().as_str(),
            "Result: 3"
        );
        assert_eq!(
            p.safe_eval(r#" $name = "Radek";"Hello $name" "#)
                .unwrap()
                .as_str(),
            "Hello Radek"
        );
        assert_eq!(
            p.safe_eval(r#" "This is a quote: `"" "#).unwrap().as_str(),
            "This is a quote: \""
        );
        assert_eq!(
            p.safe_eval(r#" "A backtick `` and escaped quote `"" "#)
                .unwrap()
                .as_str(),
            "A backtick ` and escaped quote \""
        );
        assert_eq!(
            p.safe_eval(
                r#" @"
Hello $name $
Multiline with $(1+2)
"@ "#
            )
            .unwrap()
            .as_str(),
            "Hello Radek $\nMultiline with 3"
        );

        assert_eq!(
            p.safe_eval(
                r#" @'
This is a
multi-line
here string
'@ "#
            )
            .unwrap()
            .as_str(),
            "This is a\nmulti-line\nhere string"
        );
    }
}
