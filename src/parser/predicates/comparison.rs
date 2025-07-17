use std::{collections::HashMap, sync::LazyLock};

use regex::Regex;

use super::Val;

pub(crate) type CompPredType = fn(Val, b: Val) -> bool;

pub(crate) struct ComparisonPred;

impl ComparisonPred {
    const COMP_PRED_MAP: LazyLock<HashMap<&'static str, CompPredType>> =
        LazyLock::new(|| HashMap::from([
            ("-eq", ieq as _),
            ("-ceq", ceq as _),
            ("-ieq", ieq as _),
            ("-ne", ine as _),
            ("-cne", cne as _),
            ("-ine", ine as _),
            // ("-cge", cge as _),
            // ("-ige", ige as _),
            // ("-cgt", cgt as _),
            // ("-igt", igt as _),
            // ("-cle", cle as _),
            // ("-ile", ile as _),
            // ("-contains", icontains as _),
            // ("-icontains", icontains as _),
            // ("-ccontains", ccontains as _),
            // ("-notcontains", inotcontains as _),
            // ("-inotcontains", inotcontains as _),
            // ("-cnotcontains", cnotcontains as _),
            // ("-match", imatch as _),
            // ("-imatch", imatch as _),
            // ("-cmatch", cmatch as _),
            // ("-notmatch", inotmatch as _),
            // ("-inotmatch", inotmatch as _),
            // ("-cnotmatch", cnotmatch as _),
            // ("-like", ilike as _),
            // ("-ilike", ilike as _),
            // ("-clike", clike as _),
            // ("-notlike", inotlike as _),
            // ("-inotlike", inotlike as _),
            // ("-cnotlike", cnotlike as _),
    ]));

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


/// Case-sensitive greater than or equal
fn cge(a: &str, b: &str) -> bool {
    a >= b
}

/// Case-insensitive greater than or equal
fn ige(a: &str, b: &str) -> bool {
    a.to_lowercase() >= b.to_lowercase()
}

/// Case-sensitive greater than
fn cgt(a: &str, b: &str) -> bool {
    a > b
}

/// Case-insensitive greater than
fn igt(a: &str, b: &str) -> bool {
    a.to_lowercase() > b.to_lowercase()
}

/// Case-sensitive less than or equal
fn cle(a: &str, b: &str) -> bool {
    a <= b
}

/// Case-insensitive less than or equal
fn ile(a: &str, b: &str) -> bool {
    a.to_lowercase() <= b.to_lowercase()
}

/// Case-sensitive less than
fn clt(a: &str, b: &str) -> bool {
    a < b
}

/// Case-insensitive less than
fn ilt(a: &str, b: &str) -> bool {
    a.to_lowercase() < b.to_lowercase()
}

/// Case-sensitive match (regex)
fn cmatch(a: &str, pattern: &str) -> bool {
    Regex::new(pattern).map(|re| re.is_match(a)).unwrap_or(false)
}

/// Case-insensitive match (regex)
fn imatch(a: &str, pattern: &str) -> bool {
    Regex::new(&format!("(?i){}", pattern))
        .map(|re| re.is_match(a))
        .unwrap_or(false)
}

/// Case-sensitive not match
fn cnotmatch(a: &str, pattern: &str) -> bool {
    !cmatch(a, pattern)
}

/// Case-insensitive not match
fn inotmatch(a: &str, pattern: &str) -> bool {
    !imatch(a, pattern)
}

/// Case-sensitive like (simple wildcard: * and ?)
fn clike(a: &str, pattern: &str) -> bool {
    let regex_pattern = wildcard_to_regex(pattern, false);
    Regex::new(&regex_pattern)
        .map(|re| re.is_match(a))
        .unwrap_or(false)
}

/// Case-insensitive like
fn ilike(a: &str, pattern: &str) -> bool {
    let regex_pattern = wildcard_to_regex(pattern, true);
    Regex::new(&regex_pattern)
        .map(|re| re.is_match(a))
        .unwrap_or(false)
}

/// Case-sensitive not like
fn cnotlike(a: &str, pattern: &str) -> bool {
    !clike(a, pattern)
}

/// Case-insensitive not like
fn inotlike(a: &str, pattern: &str) -> bool {
    !ilike(a, pattern)
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
        assert_eq!(p.evaluate_last_exp("1 -eq 1").unwrap(), "True".to_string());
        assert_eq!(p.evaluate_last_exp("1 -eq 2").unwrap(), "False".to_string());
        assert_eq!(p.evaluate_last_exp("\"1\" -ieq 1").unwrap(), "True".to_string());
        assert_eq!(p.evaluate_last_exp("\"A\" -ieq \"a\"").unwrap(), "True".to_string());
        assert_eq!(p.evaluate_last_exp("\"A\" -ceq \"a\"").unwrap(), "False".to_string());
        assert_eq!(p.evaluate_last_exp("\"A\" -ne \"a\"").unwrap(), "False".to_string());
        assert_eq!(p.evaluate_last_exp("\"A\" -ine \"a\"").unwrap(), "False".to_string());
        assert_eq!(p.evaluate_last_exp("\"A\" -cne \"a\"").unwrap(), "True".to_string());
    }
}