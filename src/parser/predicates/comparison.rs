use std::{collections::HashMap, sync::LazyLock};

use regex::Regex;

use super::Val;

pub(crate) type CompPredType = fn(Val, b: Val) -> bool;

pub(crate) struct ComparisonPred;

impl ComparisonPred {
    const COMP_PRED_MAP: LazyLock<HashMap<&'static str, CompPredType>> = LazyLock::new(|| {
        HashMap::from([
            ("-eq", ieq as _),
            ("-ieq", ieq as _),
            ("-ceq", ceq as _),
            ("-ne", ine as _),
            ("-ine", ine as _),
            ("-cne", cne as _),
            ("-gt", igt as _),
            ("-igt", igt as _),
            ("-cgt", cgt as _),
            ("-ge", ige as _),
            ("-ige", ige as _),
            ("-cge", cge as _),
            ("-lt", ilt as _),
            ("-ilt", ilt as _),
            ("-clt", clt as _),
            ("-le", ile as _),
            ("-ile", ile as _),
            ("-cle", cle as _),
            ("-match", imatch as _),
            ("-imatch", imatch as _),
            ("-cmatch", cmatch as _),
            ("-notmatch", inotmatch as _),
            ("-inotmatch", inotmatch as _),
            ("-cnotmatch", cnotmatch as _),
            ("-like", ilike as _),
            ("-ilike", ilike as _),
            ("-clike", clike as _),
            ("-notlike", inotlike as _),
            ("-inotlike", inotlike as _),
            ("-cnotlike", cnotlike as _),
        ])
    });

    pub(crate) fn get(name: &str) -> Option<CompPredType> {
        Self::COMP_PRED_MAP.get(name).map(|elem| *elem)
    }
}

fn eq_imp(a: Val, b: Val, case_insensitive: bool) -> bool {
    match a.eq(b, case_insensitive) {
        Ok(b) => b,
        Err(err) => {
            log::warn!("{err}");
            false
        }
    }
}

/// Case-sensitive equality
fn ceq(a: Val, b: Val) -> bool {
    eq_imp(a, b, false)
}

/// Case-insensitive equality
fn ieq(a: Val, b: Val) -> bool {
    eq_imp(a, b, true)
}

/// Case-sensitive not equal
fn cne(a: Val, b: Val) -> bool {
    !ceq(a, b)
}

/// Case-insensitive not equal
fn ine(a: Val, b: Val) -> bool {
    !ieq(a, b)
}

fn gt_imp(a: Val, b: Val, case_insensitive: bool) -> bool {
    match a.gt(b, case_insensitive) {
        Ok(b) => b,
        Err(err) => {
            log::warn!("{err}");
            false
        }
    }
}

fn lt_imp(a: Val, b: Val, case_insensitive: bool) -> bool {
    match a.lt(b, case_insensitive) {
        Ok(b) => b,
        Err(err) => {
            log::warn!("{err}");
            false
        }
    }
}

/// Case-insensitive greater than
fn igt(a: Val, b: Val) -> bool {
    gt_imp(a, b, true)
}

/// Case-sensitive greater than
fn cgt(a: Val, b: Val) -> bool {
    gt_imp(a, b, false)
}

/// Case-insensitive greater than or equal
fn ige(a: Val, b: Val) -> bool {
    !lt_imp(a, b, true)
}

/// Case-sensitive greater than or equal
fn cge(a: Val, b: Val) -> bool {
    !lt_imp(a, b, false)
}

/// Case-insensitive less than
fn ilt(a: Val, b: Val) -> bool {
    lt_imp(a, b, true)
}

/// Case-sensitive less than
fn clt(a: Val, b: Val) -> bool {
    lt_imp(a, b, false)
}

/// Case-insensitive less than or equal
fn ile(a: Val, b: Val) -> bool {
    !gt_imp(a, b, true)
}

/// Case-sensitive less than or equal
fn cle(a: Val, b: Val) -> bool {
    !gt_imp(a, b, false)
}

/// Case-sensitive match (regex)
fn cmatch(input: Val, pattern: Val) -> bool {
    Regex::new(&pattern.cast_to_string())
        .map(|re| re.is_match(&input.cast_to_string()))
        .unwrap_or(false)
}

/// Case-insensitive match (regex)
fn imatch(input: Val, pattern: Val) -> bool {
    Regex::new(&format!("(?i){}", pattern.cast_to_string()))
        .map(|re| re.is_match(&input.cast_to_string()))
        .unwrap_or(false)
}

/// Case-sensitive not match
fn cnotmatch(input: Val, pattern: Val) -> bool {
    !cmatch(input, pattern)
}

/// Case-insensitive not match
fn inotmatch(input: Val, pattern: Val) -> bool {
    !imatch(input, pattern)
}

/// Case-sensitive like (simple wildcard: * and ?)
fn clike(input: Val, pattern: Val) -> bool {
    let regex_pattern = wildcard_to_regex(&pattern.cast_to_string(), false);
    Regex::new(&regex_pattern)
        .map(|re| re.is_match(&input.cast_to_string()))
        .unwrap_or(false)
}

/// Case-insensitive like
fn ilike(input: Val, pattern: Val) -> bool {
    let regex_pattern = wildcard_to_regex(&pattern.cast_to_string(), true);
    Regex::new(&regex_pattern)
        .map(|re| re.is_match(&input.cast_to_string()))
        .unwrap_or(false)
}

/// Case-sensitive not like
fn cnotlike(input: Val, pattern: Val) -> bool {
    !clike(input, pattern)
}

/// Case-insensitive not like
fn inotlike(input: Val, pattern: Val) -> bool {
    !ilike(input, pattern)
}

/// Helper: convert wildcard pattern (*, ?) to regex pattern.
/// if case_insensitive is true, add `(?i)` prefix.
fn wildcard_to_regex(pattern: &str, case_insensitive: bool) -> String {
    let mut regex = String::new();
    if case_insensitive {
        regex.push_str("(?i)");
    }
    regex.push('^');
    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' | '#' | '-' => {
                regex.push('\\');
                regex.push(ch);
            }
            _ => regex.push(ch),
        }
    }
    regex.push('$');
    regex
}

#[cfg(test)]
mod tests {
    use crate::PowerShellParser;

    #[test]
    fn test_eq() {
        let mut p = PowerShellParser::new();
        assert_eq!(p.safe_eval("1 -eq 1").unwrap(), "True".to_string());
        assert_eq!(p.safe_eval("1 -eq 2").unwrap(), "False".to_string());
        assert_eq!(
            p.safe_eval("\"1\" -ieq 1").unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval("\"A\" -ieq \"a\"").unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval("\"A\" -ceq \"a\"").unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval("\"A\" -ne \"a\"").unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval("\"A\" -ine \"a\"").unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval("\"A\" -cne \"a\"").unwrap(),
            "True".to_string()
        );
    }

    #[test]
    fn test_gt() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(r#"2 -gt 1"#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#"[char]1 -le "b""#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "c" -ge [char]99 "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "b" -ge [char]99 "#).unwrap(),
            "False".to_string()
        );

        assert_eq!(
            p.safe_eval(r#" "a" -gt "A" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "a" -igt "A" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "A" -cgt "A" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "A" -cgt "a" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "a" -lt "A" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "A" -ilt "a" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "a" -clt "A" "#).unwrap(),
            "True".to_string()
        );

        assert_eq!(
            p.safe_eval(r#" "a" -ge "A" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "a" -ige "A" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "a" -cge "A" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "A" -cge "A" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "A" -le "a" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "A" -ile "a" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "A" -cle "a" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "A" -cle "A" "#).unwrap(),
            "True".to_string()
        );
    }

    #[test]
    fn test_match() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(r#" "Hello World" -match "hello" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -imatch "hello" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -cmatch "hello" "#)
                .unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -cnotmatch "hello" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "abc123xyz" -cmatch "\d{3}" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "abc123xyz" -cmatch 123 "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "anything" -cmatch "" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "user@example.com" -cmatch "\w+@\w+\.\w+" "#)
                .unwrap(),
            "True".to_string()
        );
    }

    #[test]
    fn test_like() {
        let mut p = PowerShellParser::new();
        assert_eq!(
            p.safe_eval(r#" "Hello World" -like "hello*" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -ilike "hello*" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -clike "hello*" "#)
                .unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -clike "Hello*" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -cnotlike "hello*" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -clike "*llo*" "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "Hello World" -cnotlike "*lllo*" "#)
                .unwrap(),
            "True".to_string()
        );

        assert_eq!(
            p.safe_eval(r#" "cat" -clike "c?t" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "cut" -clike "c?t" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "coat" -notlike "c?t" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "CUt" -cnotlike "c?t" "#).unwrap(),
            "True".to_string()
        );
    }
}
