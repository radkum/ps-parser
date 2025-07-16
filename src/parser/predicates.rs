use std::{collections::HashMap, sync::LazyLock};

use super::Val;

fn add(mut arg1: Val, arg2: Val) -> Val {
    arg1.add(arg2);
    arg1
}

fn sub(mut arg1: Val, arg2: Val) -> Val {
    arg1.sub(arg2);
    arg1
}

fn mul(arg1: Val, arg2: Val) -> Val {
    match (arg1, arg2) {
        (Val::Int(i1), Val::Int(i2)) => Val::Int(i1 * i2),
        _ => panic!(),
    }
}

fn div(arg1: Val, arg2: Val) -> Val {
    match (arg1, arg2) {
        (Val::Int(i1), Val::Int(i2)) => Val::Int(i1 / i2),
        _ => panic!(),
    }
}

fn modulo(arg1: Val, arg2: Val) -> Val {
    match (arg1, arg2) {
        (Val::Int(i1), Val::Int(i2)) => Val::Int(i1 % i2),
        _ => panic!(),
    }
}

fn assign(_arg1: Val, arg2: Val) -> Val {
    arg2
}

pub(crate) type PredType = fn(Val, Val) -> Val;

pub(crate) struct Predicates;

impl Predicates {
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
