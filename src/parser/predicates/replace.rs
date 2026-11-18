use std::{collections::HashMap, sync::LazyLock};

use regex::{Captures, Regex};

use super::Val;

pub(crate) type ReplacePredType = fn(Val, Val, Val) -> String;

pub(crate) struct ReplacePred;

impl ReplacePred {
    const REPLACE_PRED_MAP: LazyLock<HashMap<&'static str, ReplacePredType>> =
        LazyLock::new(|| {
            HashMap::from([
                ("-replace", ireplace as _),
                ("-ireplace", ireplace as _),
                ("-creplace", creplace as _),
            ])
        });

    pub(crate) fn get(name: &str) -> Option<ReplacePredType> {
        Self::REPLACE_PRED_MAP.get(name).copied()
    }
}

pub fn ireplace(input: Val, pattern: Val, replacement: Val) -> String {
    let ci_pattern = format!("(?i){}", pattern.cast_to_string()); // make regex case-insensitive
    match Regex::new(&ci_pattern) {
        Ok(re) => replace_all(&re, input, replacement),
        Err(_) => input.cast_to_string(),
    }
}

fn creplace(input: Val, pattern: Val, replacement: Val) -> String {
    match Regex::new(pattern.cast_to_string().as_str()) {
        Ok(re) => replace_all(&re, input, replacement),
        Err(_) => input.cast_to_string(), // fallback: return input unchanged on invalid regex
    }
}

// expands matches one by one so a long replacement can't exhaust memory
fn replace_all(re: &Regex, input: Val, replacement: Val) -> String {
    let replacement = replacement.cast_to_string();
    re.replace_all(input.cast_to_string().as_str(), |caps: &Captures| {
        let mut dst = String::new();
        caps.expand(&replacement, &mut dst);
        if !crate::parser::charge(dst.len()) {
            dst.clear();
        }
        dst
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use crate::PowerShellSession;

    #[test]
    fn test_replace() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(r#""Hello World" -replace "World", "PowerShell""#)
                .unwrap(),
            "Hello PowerShell".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "abc123" -replace "\d+", "456" "#).unwrap(),
            "abc456".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "one two One two" -rEplace "one", "1" "#)
                .unwrap(),
            "1 two 1 two".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "one two One two" -ireplace "one", "1" "#)
                .unwrap(),
            "1 two 1 two".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "one two One two" -crEplace "one", "1" "#)
                .unwrap(),
            "1 two One two".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Color colour" -replace "(?i)colou?r", "paint" "#)
                .unwrap(),
            "paint paint".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Color colour" -ireplace "(?i)colou?r", "paint" "#)
                .unwrap(),
            "paint paint".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Color colour" -creplace "(?i)colou?r", "paint" "#)
                .unwrap(),
            "paint paint".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "1+1=2" -replace "\+", " plus " "#).unwrap(),
            "1 plus 1=2".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Power  Shell" -replace "\s+", "_" "#)
                .unwrap(),
            "Power_Shell".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "abc123def456" -replace "\d", "" "#)
                .unwrap(),
            "abcdef".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" [int]("e1e2" -replace "e") * 8 "#).unwrap(),
            "96".to_string()
        );
    }
}
