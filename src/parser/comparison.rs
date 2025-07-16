use std::{collections::HashMap, sync::LazyLock};

use super::Val;

pub(crate) type CompPredType = fn(Val, Vec<Val>) -> Val;

pub(crate) struct Comparison;

fn replace(mut s: Val, args: Vec<Val>) -> Val {
    if args.len() == 2 {
        if let (Val::String(s), Val::String(original), Val::String(new)) =
            (&mut s, args[0].clone(), args[1].clone())
        {
            *s = s.replace(original.as_str(), new.as_str());
        }
    }
    s
}

impl Comparison {
    const COMPARISON_PRED_MAP: LazyLock<HashMap<&'static str, CompPredType>> =
        LazyLock::new(|| HashMap::from([("-replace", replace as _)]));

    pub(crate) fn get(name: &str) -> Option<CompPredType> {
        Self::COMPARISON_PRED_MAP.get(name).map(|elem| *elem)
    }
}

/*
Equality

-eq, -ieq, -ceq - equals
-ne, -ine, -cne - not equals
-gt, -igt, -cgt - greater than
-ge, -ige, -cge - greater than or equal
-lt, -ilt, -clt - less than
-le, -ile, -cle - less than or equal
Matching

-like, -ilike, -clike - string matches wildcard pattern
-notlike, -inotlike, -cnotlike - string doesn't match wildcard pattern
-match, -imatch, -cmatch - string matches regex pattern
-notmatch, -inotmatch, -cnotmatch - string doesn't match regex pattern
Replacement

-replace, -ireplace, -creplace - finds and replaces strings matching a regex pattern
Containment

-contains, -icontains, -ccontains - collection contains a value
-notcontains, -inotcontains, -cnotcontains - collection doesn't contain a value
-in, -iin, -cin - value is in a collection
-notin, -inotin, -cnotin - value isn't in a collection
Type

-is - both objects are the same type
-isnot - the objects aren't the same type
*/
