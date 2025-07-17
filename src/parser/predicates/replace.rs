
use super::Val;
use std::{collections::HashMap, sync::LazyLock};
use regex::Regex;

pub(crate) type ReplacePredType = fn(Val, Val, Val) -> String;

pub(crate) struct ReplacePred;

impl ReplacePred {
    const REPLACE_PRED_MAP: LazyLock<HashMap<&'static str, ReplacePredType>> =
        LazyLock::new(|| HashMap::from([
            ("-replace", ireplace as _),
            ("-ireplace", ireplace as _),
            ("-creplace", creplace as _),
            ]));

    pub(crate) fn get(name: &str) -> Option<ReplacePredType> {
        Self::REPLACE_PRED_MAP.get(name).map(|elem| *elem)
    }
}

pub fn ireplace(input: Val, pattern: Val, replacement: Val) -> String {
    let ci_pattern = format!("(?i){}", pattern.cast_to_string()); // make regex case-insensitive
    match Regex::new(&ci_pattern) {
        Ok(re) => re.replace_all(input.cast_to_string().as_str(), replacement.cast_to_string()).to_string(),
        Err(_) => input.cast_to_string(),
    }
}

fn creplace(input: Val, pattern: Val, replacement: Val) -> String {
    match Regex::new(pattern.cast_to_string().as_str()) {
        Ok(re) => re.replace_all(input.cast_to_string().as_str(), replacement.cast_to_string()).to_string(),
        Err(_) => input.cast_to_string(), // fallback: return input unchanged on invalid regex
    }
}

#[cfg(test)]
mod tests {
    use crate::PowerShellParser;

    #[test]
    fn test_replace() {
        let mut p = PowerShellParser::new();
        assert_eq!(p.evaluate_last_exp(r#""Hello World" -replace "World", "PowerShell""#).unwrap(), "Hello PowerShell".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "abc123" -replace "\d+", "456" "#).unwrap(), "abc456".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "one two One two" -replace "one", "1" "#).unwrap(), "1 two 1 two".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "one two One two" -ireplace "one", "1" "#).unwrap(), "1 two 1 two".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "one two One two" -creplace "one", "1" "#).unwrap(), "1 two One two".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "Color colour" -replace "(?i)colou?r", "paint" "#).unwrap(), "paint paint".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "Color colour" -ireplace "(?i)colou?r", "paint" "#).unwrap(), "paint paint".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "Color colour" -creplace "(?i)colou?r", "paint" "#).unwrap(), "paint paint".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "1+1=2" -replace "\+", " plus " "#).unwrap(), "1 plus 1=2".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "Power  Shell" -replace "\s+", "_" "#).unwrap(), "Power_Shell".to_string());
        assert_eq!(p.evaluate_last_exp(r#" "abc123def456" -replace "\d", "" "#).unwrap(), "abcdef".to_string());
    }
}