use std::{collections::HashMap, sync::LazyLock};

use super::Val;

pub(crate) type ContainPredType = fn(Val, Val) -> bool;

pub(crate) struct ContainPred;

impl ContainPred {
    const LOGICAL_PRED_MAP: LazyLock<HashMap<&'static str, ContainPredType>> =
        LazyLock::new(|| {
            HashMap::from([
                ("-in", iin as _),
                ("-iin", iin as _),
                ("-cin", cin as _),
                ("-notin", inotin as _),
                ("-inotin", inotin as _),
                ("-cnotin", cnotin as _),
                ("-contains", icontains as _),
                ("-icontains", icontains as _),
                ("-ccontains", ccontains as _),
                ("-notcontains", inotcontains as _),
                ("-inotcontains", inotcontains as _),
                ("-cnotcontains", cnotcontains as _),
            ])
        });

    pub(crate) fn get(name: &str) -> Option<ContainPredType> {
        Self::LOGICAL_PRED_MAP.get(name).map(|elem| *elem)
    }
}

fn contains_impl(a: Val, b: Val, case_insensitive: bool) -> bool {
    let mut array = if let Val::Array(box_vec) = a {
        box_vec
            .iter()
            .map(|v| v.cast_to_string())
            .collect::<Vec<String>>()
    } else {
        vec![a.cast_to_string()]
    };
    let mut elem = b.cast_to_string();

    if case_insensitive {
        array = array
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect::<Vec<String>>();
        elem = elem.to_ascii_lowercase();
    }

    array.contains(&elem)
}

pub fn iin(a: Val, b: Val) -> bool {
    contains_impl(b, a, true)
}

pub fn cin(a: Val, b: Val) -> bool {
    contains_impl(b, a, false)
}

pub fn inotin(a: Val, b: Val) -> bool {
    !iin(a, b)
}

pub fn cnotin(a: Val, b: Val) -> bool {
    !cin(a, b)
}

pub fn icontains(a: Val, b: Val) -> bool {
    contains_impl(a, b, true)
}

pub fn ccontains(a: Val, b: Val) -> bool {
    contains_impl(a, b, false)
}

pub fn inotcontains(a: Val, b: Val) -> bool {
    !icontains(a, b)
}

pub fn cnotcontains(a: Val, b: Val) -> bool {
    !ccontains(a, b)
}

#[cfg(test)]
mod tests {
    use crate::PowerShellSession;

    #[test]
    fn test_in() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(r#" ($true, 1) -in ("True 1", 2)  "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" ($true, 1) -in (("True1", "1"), 2)  "#)
                .unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" 1 -in "3", "1"   "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(p.safe_eval(r#" 1 -in "1"   "#).unwrap(), "True".to_string());
        assert_eq!(
            p.safe_eval(r#" $true -iIn "true", "1" "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $true -cin "true", "1" "#).unwrap(),
            "False".to_string()
        );
    }

    #[test]
    fn test_notin() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(r#" ($true, 1) -notin ("True 1", 2)  "#)
                .unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" ($true, 1) -inotin (("True1", "1"), 2) "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $true -notin "true", "1" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $true -inotin "true", "1" "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" $true -cnotin "true", "1" "#).unwrap(),
            "True".to_string()
        );
    }

    #[test]
    fn test_constains() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(r#" ("True 1", 2) -Contains ($true, 1) "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" (("True1", "1"), 2) -contains ($true, 1) "#)
                .unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "3", "1" -contains 1 "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "1" -contains 1 "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "true", "1" -icontains $true "#).unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "true", "1" -ccontains $true "#).unwrap(),
            "False".to_string()
        );
    }

    #[test]
    fn test_notconstains() {
        let mut p = PowerShellSession::new();
        assert_eq!(
            p.safe_eval(r#" ("True 1", 2) -notcontains ($true, 1) "#)
                .unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" (("True1", "1"), 2) -notcontains ($true, 1) "#)
                .unwrap(),
            "True".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "3", "1" -notcontains 1 "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "1" -notcontains 1 "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "true", "1" -notcontains $true "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "true", "1" -inotcontains $true "#).unwrap(),
            "False".to_string()
        );
        assert_eq!(
            p.safe_eval(r#" "true", "1" -cnotcontains $true "#).unwrap(),
            "True".to_string()
        );
    }
}
