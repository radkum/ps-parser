use std::{collections::HashMap, sync::LazyLock};

use regex::RegexBuilder;

use super::{Val, ValType};
pub(crate) type SplitPredType = fn(Val, Val) -> Val;

pub(crate) struct SplitPred;

impl SplitPred {
    const JOIN_PRED_MAP: LazyLock<HashMap<&'static str, SplitPredType>> = LazyLock::new(|| {
        HashMap::from([
            ("-split", isplit as _),
            ("-isplit", isplit as _),
            ("-csplit", csplit as _),
        ])
    });

    pub(crate) fn get(name: &str) -> Option<SplitPredType> {
        Self::JOIN_PRED_MAP.get(name).copied()
    }
}

pub fn powershell_split(
    input: &str,
    pattern: Option<String>,
    max_splits: Option<usize>,
    case_insensitive: bool,
) -> Result<Vec<String>, regex::Error> {
    let pattern = pattern.unwrap_or(r"\s+".to_string());

    if pattern.starts_with("(") && pattern.ends_with(")") {
        let Some(pat) = pattern
            .strip_prefix("(")
            .unwrap_or_default()
            .strip_suffix(")")
        else {
            return Ok(vec![]);
        };
        return powershell_split_preserve_delimeter(
            input,
            pat.to_string(),
            max_splits,
            case_insensitive,
        );
    }

    let re = RegexBuilder::new(&pattern)
        .case_insensitive(case_insensitive)
        .build()?;

    let result = if let Some(limit) = max_splits {
        re.splitn(input, limit).map(|s| s.to_string()).collect()
    } else {
        re.split(input).map(|s| s.to_string()).collect()
    };

    Ok(result)
}

pub fn powershell_split_preserve_delimeter(
    input: &str,
    pattern: String,
    max_splits: Option<usize>,
    case_insensitive: bool,
) -> Result<Vec<String>, regex::Error> {
    let re = RegexBuilder::new(&pattern)
        .case_insensitive(case_insensitive)
        .build()?;

    let mut result = Vec::new();
    let mut last_end = 0;

    for (splits, mat) in re.find_iter(input).enumerate() {
        if let Some(limit) = max_splits {
            if splits >= limit {
                break;
            }
        }

        // Push text before the match
        if last_end < mat.start() {
            result.push(input[last_end..mat.start()].to_string());
        }

        // Push the delimiter itself
        result.push(mat.as_str().to_string());

        last_end = mat.end();
    }

    // Push the remaining part of the string
    if last_end < input.len() {
        result.push(input[last_end..].to_string());
    }

    Ok(result)
}

/// -split operator (case-sensitive)
pub fn split(input: Val, args: Val, case_insensitive: bool) -> Val {
    //special case when, input is Val::Null, eg. "-split 'ad fa'"
    let (input, args) = if input.ttype() == ValType::Null {
        // let Val::Array(box_vec) = args else {
        //     return Val::Null
        // };
        (args, vec![])
    } else {
        (input, args.cast_to_array())
    };

    log::trace!("input: {:?}", input);
    let mut pattern = None;
    let mut max_splits = None;

    if !args.is_empty() {
        pattern = Some(args[0].cast_to_string())
    }

    if args.len() > 1 {
        let Ok(splits) = args[1].cast_to_int() else {
            return Val::Null;
        };
        max_splits = Some(splits as usize);
    }

    let mut res = vec![];
    let input_array = input.cast_to_array();
    for i in input_array.into_iter() {
        if let Ok(v) = powershell_split(
            &i.cast_to_string(),
            pattern.clone(),
            max_splits,
            case_insensitive,
        ) {
            res.push(Val::String(v.join(" ").into()))
        }
    }
    if res.is_empty() {
        Val::Null
    } else if res.len() == 1 {
        res[0].clone()
    } else {
        Val::Array(res)
    }
}

/// -isplit operator (case-insensitive)
pub fn isplit(input: Val, args: Val) -> Val {
    log::trace!("split: {:?} {:?}", input, args);
    split(input, args, true)
}

/// -csplit operator (case-sensitive)
pub fn csplit(input: Val, args: Val) -> Val {
    split(input, args, false)
}

#[cfg(test)]
mod tests {
    use crate::PowerShellParser;

    #[test]
    fn test_split() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(r#" -sPlit "red yellow blue green" "#).unwrap(),
            "red yellow blue green".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -split ("red", "yellow blue green") "#)
                .unwrap(),
            "red yellow blue green".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -split ("red", "yellow blue green"), 2 "#)
                .unwrap(),
            "red yellow blue green 2".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -split @("red", "yellow blue green") "#)
                .unwrap(),
            "red yellow blue green".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" -split @("red", "yellow blue green"), 2 "#)
                .unwrap(),
            "red yellow blue green 2".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Lastname:FirstName:Address" -split ":" "#)
                .unwrap(),
            "Lastname FirstName Address".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Lastname:FirstName:Address" -split "(:)" "#)
                .unwrap(),
            "Lastname : FirstName : Address".to_string()
        );
        assert_eq!(PowerShellParser::new().safe_eval(r#" $c = "Mercury,Venus,Earth,Mars,Jupiter,Saturn,Uranus,Neptune";$c -split ",", 5 "#).unwrap(),"Mercury Venus Earth Mars Jupiter,Saturn,Uranus,Neptune".to_string());
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" [string] (-isplit @('a,b c','1 2,3,4,5', '5,6,7,8')) "#)
                .unwrap(),
            "a,b c 1 2,3,4,5 5,6,7,8".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" $c = 'a,b,c','1,2,3,4,5', '5,6,7,8';[string]($c -split ',', 2) "#)
                .unwrap(),
            "a b,c 1 2,3,4,5 5 6,7,8".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" $c = 2121212, 1212;[string]($c -split '1', 2) "#)
                .unwrap(),
            "2 21212  212".to_string()
        );
        assert_eq!(
            PowerShellParser::new()
                .safe_eval(r#" [string]("Mercury,Venus,Earth" -split '[et]')  "#)
                .unwrap(),
            "M rcury,V nus, ar h".to_string()
        );
        assert_eq!(PowerShellParser::new().safe_eval(r#" $c = "Mercury,Venus,Earth,Mars,Jupiter,Saturn,Uranus,Neptune";[string]($c -split {$_ -eq "e" -or $_ -eq "p"}) "#).unwrap(),"M rcury,V nus, arth,Mars,Ju it r,Saturn,Uranus,N  tun".to_string());
    }
}
