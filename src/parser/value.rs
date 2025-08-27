mod method_error;
mod ps_string;
mod runtime_object;
mod system_convert;
mod system_encoding;
mod val_error;

use std::{collections::HashMap, ops::Neg, sync::LazyLock};

pub(crate) use method_error::{MethodError, MethodResult};
pub(crate) use ps_string::PsString;
use ps_string::str_cmp;
pub(super) use runtime_object::RuntimeObject;
use runtime_object::{MethodCallType, StaticFnCallType};
use smart_default::SmartDefault;
use system_convert::Convert;
use system_encoding::Encoding;
pub(crate) use val_error::ValError;
pub type ValResult<T> = core::result::Result<T, ValError>;

#[derive(PartialEq, Debug)]
pub enum ValType {
    Null,
    Bool,
    Int,
    Float,
    Char,
    String,
    Array,
    RuntimeType(String),
}

impl std::fmt::Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ValType {
    const STATIC_OBJECT_MAP: LazyLock<HashMap<&'static str, Box<dyn RuntimeObject>>> =
        LazyLock::new(|| {
            HashMap::from([
                ("system.convert", Box::new(Convert {}) as _),
                ("system.text.encoding", Box::new(Encoding {}) as _),
            ])
        });

    pub(crate) fn cast(s: &str) -> ValResult<Self> {
        let s = s.to_ascii_lowercase();
        let t = match s.as_str() {
            "char" | "byte" => Self::Char,
            "bool" => Self::Bool,
            "int" | "long" | "decimal" => Self::Int,
            "float" | "double" => Self::Float,
            "string" => Self::String,
            "array" => Self::Array,
            _ => {
                if !Self::STATIC_OBJECT_MAP.contains_key(s.as_str()) {
                    Err(ValError::UnknownType(s.clone()))?;
                }

                Self::RuntimeType(s)
            }
        };
        Ok(t)
    }
}

#[derive(Debug, SmartDefault)]
pub(crate) enum Val {
    #[default]
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(u32),
    String(PsString),
    Array(Vec<Val>),
    RuntimeObject(Box<dyn RuntimeObject>),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cast_to_string())
    }
}

impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Val::Null, Val::Null) => true,
            (Val::Bool(a), Val::Bool(b)) => a == b,
            (Val::Int(a), Val::Int(b)) => a == b,
            (Val::Float(a), Val::Float(b)) => a == b,
            (Val::Char(a), Val::Char(b)) => a == b,
            (Val::String(a), Val::String(b)) => a == b,
            (Val::Array(a), Val::Array(b)) => a == b,
            (Val::RuntimeObject(a), Val::RuntimeObject(b)) => a.name() == b.name(),
            _ => false,
        }
    }
}

impl Clone for Val {
    fn clone(&self) -> Self {
        match self {
            Val::Null => Val::Null,
            Val::Bool(a) => Val::Bool(*a),
            Val::Int(a) => Val::Int(*a),
            Val::Float(a) => Val::Float(*a),
            Val::Char(a) => Val::Char(*a),
            Val::String(a) => Val::String(a.clone()),
            Val::Array(a) => Val::Array(a.clone()),
            Val::RuntimeObject(a) => {
                if let Ok(runtime_object) = runtime_object::get_runtime_object(a.name().as_str()) {
                    Val::RuntimeObject(runtime_object)
                } else {
                    Val::Null
                }
            }
        }
    }
}

impl Val {
    pub fn eq(&self, val: Val, case_insensitive: bool) -> ValResult<bool> {
        Ok(match self {
            Val::Null => val.ttype() == ValType::Null,
            Val::Bool(b) => *b == val.cast_to_bool(),
            Val::Char(c) => *c == val.cast_to_char()?,
            Val::Int(i) => *i == val.cast_to_int()?,
            Val::Float(f) => *f == val.cast_to_float()?,
            Val::String(PsString(s1)) => {
                let s2 = val.cast_to_string();
                str_cmp(s1, &s2, case_insensitive) == std::cmp::Ordering::Equal
            }
            Val::Array(_) => todo!(),
            Val::RuntimeObject(s) => {
                if let Val::RuntimeObject(s2) = val {
                    str_cmp(&s.name(), &s2.name(), case_insensitive) == std::cmp::Ordering::Equal
                } else {
                    false
                }
            }
        })
    }

    pub fn gt(&self, val: Val, case_insensitive: bool) -> ValResult<bool> {
        Ok(match self {
            Val::Null => false,
            Val::Bool(b) => *b & !val.cast_to_bool(),
            Val::Char(c) => *c > val.cast_to_char()?,
            Val::Int(i) => *i > val.cast_to_int()?,
            Val::Float(f) => *f > val.cast_to_float()?,
            Val::String(PsString(s1)) => {
                let s2 = val.cast_to_string();
                str_cmp(s1, &s2, case_insensitive) == std::cmp::Ordering::Greater
            }
            Val::Array(_) => todo!(),
            Val::RuntimeObject(_) => todo!(),
        })
    }

    pub fn lt(&self, val: Val, case_insensitive: bool) -> ValResult<bool> {
        Ok(match self {
            Val::Null => false,
            Val::Bool(b) => !(*b) & val.cast_to_bool(),
            Val::Char(c) => *c < val.cast_to_char()?,
            Val::Int(i) => *i < val.cast_to_int()?,
            Val::Float(f) => *f < val.cast_to_float()?,
            Val::String(PsString(s1)) => {
                let s2 = val.cast_to_string();
                str_cmp(s1, &s2, case_insensitive) == std::cmp::Ordering::Less
            }
            Val::Array(_) => todo!(),
            Val::RuntimeObject(_) => todo!(),
        })
    }

    pub fn ttype(&self) -> ValType {
        match self {
            Val::Null => ValType::Null,
            Val::Bool(_) => ValType::Bool,
            Val::Int(_) => ValType::Int,
            Val::Float(_) => ValType::Float,
            Val::Char(_) => ValType::Char,
            Val::String(_) => ValType::String,
            Val::Array(_) => ValType::Array,
            Val::RuntimeObject(_) => todo!(),
        }
    }

    pub fn add(&mut self, val: Val) -> ValResult<()> {
        match self {
            Val::Null => *self = val,
            Val::Bool(_) | Val::Int(_) | Val::Float(_) => {
                *self = if val.ttype() == ValType::Float {
                    Val::Float(self.cast_to_float()? + val.cast_to_float()?)
                } else {
                    Val::Int(self.cast_to_int()? + val.cast_to_int()?)
                };
            }
            Val::Char(_) | Val::String(_) => {
                *self = Val::String(PsString(
                    self.cast_to_string() + val.cast_to_string().as_str(),
                ))
            }
            Val::Array(arr) => arr.push(val),
            Val::RuntimeObject(_) => todo!(),
        }
        Ok(())
    }

    fn inc_or_dec_operation(&mut self, amount: i64, op: String) -> ValResult<()> {
        match self {
            Val::Null => *self = Val::Int(amount),
            Val::Int(i) => *self = Val::Int(*i + amount),
            Val::Float(f) => *self = Val::Float(*f + amount as f64),
            Val::Bool(_)
            | Val::Char(_)
            | Val::String(_)
            | Val::Array(_)
            | Val::RuntimeObject(_) => {
                //error
                Err(ValError::OperationNotDefined(
                    op,
                    self.ttype().to_string(),
                    self.ttype().to_string(),
                ))?
            }
        }
        Ok(())
    }

    pub fn inc(&mut self) -> ValResult<()> {
        self.inc_or_dec_operation(1, "++".to_string())
    }

    pub fn dec(&mut self) -> ValResult<()> {
        self.inc_or_dec_operation(-1, "--".to_string())
    }

    pub fn sub(&mut self, val: Val) -> ValResult<()> {
        if let ValType::RuntimeType(_) = self.ttype() {
            Err(ValError::OperationNotDefined(
                "-".to_string(),
                self.ttype().to_string(),
                val.ttype().to_string(),
            ))?
        }

        if let ValType::RuntimeType(_) = val.ttype() {
            Err(ValError::OperationNotDefined(
                "-".to_string(),
                self.ttype().to_string(),
                val.ttype().to_string(),
            ))?
        }

        if self.ttype() == ValType::Array || val.ttype() == ValType::Array {
            Err(ValError::OperationNotDefined(
                "-".to_string(),
                self.ttype().to_string(),
                val.ttype().to_string(),
            ))?
        }

        if self.ttype() == ValType::Float || val.ttype() == ValType::Float {
            *self = Val::Float(self.cast_to_float()? - val.cast_to_float()?);
        } else {
            *self = Val::Int(self.cast_to_int()? - val.cast_to_int()?);
        }

        Ok(())
    }

    pub fn mul(&mut self, val: Val) -> ValResult<()> {
        *self = match self {
            Val::Null => self.clone(),
            Val::Bool(_) => Err(ValError::OperationNotDefined(
                "*".to_string(),
                "Bool".to_string(),
                val.ttype().to_string(),
            ))?,
            Val::Int(_) | Val::Float(_) => {
                if self.ttype() == ValType::Float || val.ttype() == ValType::Float {
                    Val::Float(self.cast_to_float()? * val.cast_to_float()?)
                } else {
                    Val::Int(self.cast_to_int()? * val.cast_to_int()?)
                }
            }
            Val::Char(_) => Err(ValError::OperationNotDefined(
                "*".to_string(),
                "Char".to_string(),
                val.ttype().to_string(),
            ))?,
            Val::String(PsString(s)) => {
                Val::String(PsString(s.repeat(val.cast_to_int()? as usize)))
            }
            Val::Array(v) => Val::Array(Self::repeat(v, val.cast_to_int()? as usize)),
            Val::RuntimeObject(_) => {
                //error
                Err(ValError::OperationNotDefined(
                    "*".to_string(),
                    self.ttype().to_string(),
                    self.ttype().to_string(),
                ))?
            }
        };
        Ok(())
    }

    pub fn div(&mut self, val: Val) -> ValResult<()> {
        if self.ttype() == ValType::Array || val.ttype() == ValType::Array {
            Err(ValError::OperationNotDefined(
                "/".to_string(),
                self.ttype().to_string(),
                val.ttype().to_string(),
            ))?
        }

        // check dividing by zero
        if let Ok(v) = val.cast_to_float() {
            if v == 0. {
                Err(ValError::DividingByZero)?
            }
        }

        *self = match self {
            Val::Null => Val::Int(0),
            Val::Bool(_) | Val::Int(_) | Val::Char(_) | Val::String(_) => {
                //if second operand isn't float and can be divided without rest, we can cast it
                // to Int
                if val.ttype() != ValType::Float && (self.cast_to_int()? % val.cast_to_int()? == 0)
                {
                    Val::Int(self.cast_to_int()? / val.cast_to_int()?)
                } else {
                    Val::Float(self.cast_to_float()? / val.cast_to_float()?)
                }
            }
            Val::Float(_) => Val::Float(self.cast_to_float()? / self.cast_to_float()?),
            Val::Array(_) => todo!(),
            Val::RuntimeObject(_) => todo!(),
        };
        Ok(())
    }

    pub fn modulo(&mut self, val: Val) -> ValResult<()> {
        if self.ttype() == ValType::Array || val.ttype() == ValType::Array {
            Err(ValError::OperationNotDefined(
                "%".to_string(),
                self.ttype().to_string(),
                val.ttype().to_string(),
            ))?
        }

        // check dividing by zero
        if let Ok(v) = val.cast_to_float() {
            if v == 0. {
                Err(ValError::DividingByZero)?
            }
        }

        *self = match self {
            Val::Null => Val::Int(0),
            Val::Bool(_) | Val::Int(_) | Val::Char(_) | Val::String(_) => {
                //if second operand isn't float and can be divided without rest, we can cast it
                // to Int
                if val.ttype() != ValType::Float {
                    Val::Int(self.cast_to_int()? % val.cast_to_int()?)
                } else {
                    Val::Float(self.cast_to_float()? % val.cast_to_float()?)
                }
            }
            Val::Float(_) => Val::Float(self.cast_to_float()? % self.cast_to_float()?),
            Val::Array(_) => todo!(),
            Val::RuntimeObject(_) => todo!(),
        };
        Ok(())
    }

    pub fn neg(&mut self) -> ValResult<()> {
        match self {
            Val::Float(f) => *f = f.neg(),
            Val::Null | Val::Bool(_) | Val::Int(_) | Val::Char(_) | Val::String(_) => {
                *self = Val::Int(self.cast_to_int()?.neg())
            }
            Val::Array(_) => Err(ValError::OperationNotDefined(
                "-".to_string(),
                self.ttype().to_string(),
                self.ttype().to_string(),
            ))?,
            Val::RuntimeObject(_) => todo!(),
        }
        Ok(())
    }

    pub(crate) fn cast(&mut self, ttype: ValType) -> ValResult<Self> {
        Ok(match ttype {
            ValType::Null => Err(ValError::UnknownType("Null".to_string()))?,
            ValType::Bool => Val::Bool(self.cast_to_bool()),
            ValType::Int => Val::Int(self.cast_to_int()?),
            ValType::Float => Val::Float(self.cast_to_float()?),
            ValType::Char => Val::Char(self.cast_to_char()?),
            ValType::String => Val::String(PsString(self.cast_to_string())),
            ValType::Array => Val::Array(self.cast_to_array()),
            ValType::RuntimeType(_) => todo!(),
        })
    }

    pub(crate) fn init(ttype: ValType) -> ValResult<Self> {
        Ok(match ttype {
            ValType::Null => Err(ValError::UnknownType("Null".to_string()))?,
            ValType::Bool => Val::Bool(false),
            ValType::Int => Val::Int(0),
            ValType::Float => Val::Float(0.),
            ValType::Char => Val::Char(0),
            ValType::String => Val::String(PsString::default()),
            ValType::Array => Val::Array(Default::default()),
            ValType::RuntimeType(s) => {
                if let Ok(runtime_object) = runtime_object::get_runtime_object(s.as_str()) {
                    Val::RuntimeObject(runtime_object)
                } else {
                    Err(ValError::UnknownType(s.to_string()))?
                }
            }
        })
    }

    pub(crate) fn cast_to_bool(&self) -> bool {
        match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Char(c) => *c != 0,
            Val::Int(i) => *i != 0,
            Val::Float(f) => *f != 0.,
            Val::String(PsString(s)) => !s.is_empty(),
            Val::Array(v) => !v.is_empty(),
            Val::RuntimeObject(_) => todo!(),
        }
    }

    fn cast_to_char(&self) -> ValResult<u32> {
        let res = match self {
            Val::Null | Val::Int(_) | Val::Char(_) => self.cast_to_int()? as u32,
            Val::Bool(_) => Err(ValError::InvalidCast(
                "Bool".to_string(),
                "Char".to_string(),
            ))?,
            Val::Float(_) => Err(ValError::InvalidCast(
                "Float".to_string(),
                "Char".to_string(),
            ))?,
            Val::String(PsString(s)) => {
                if s.len() == 1 {
                    s.chars().next().unwrap_or_default() as u32
                } else {
                    Err(ValError::InvalidCast(
                        "String with len() more than 1".to_string(),
                        "Char".to_string(),
                    ))?
                }
            }
            Val::Array(_) => Err(ValError::InvalidCast(
                "Array".to_string(),
                "Char".to_string(),
            ))?,
            Val::RuntimeObject(_) => todo!(),
        };
        Ok(res)
    }

    pub(crate) fn cast_to_int(&self) -> ValResult<i64> {
        Ok(match self {
            Val::Null => 0,
            Val::Bool(b) => *b as i64,
            Val::Int(i) => *i,
            Val::Float(f) => f.round() as i64,
            Val::Char(c) => *c as i64,
            Val::String(PsString(s)) => {
                let s = s.to_ascii_lowercase();
                if let Some(hex) = s.strip_prefix("0x") {
                    i64::from_str_radix(hex, 16)?
                } else if let Ok(casted) = s.trim().parse::<f64>() {
                    Self::round_bankers(casted) as i64
                } else {
                    s.trim().parse::<i64>()?
                }
            }
            Val::Array(_) => Err(ValError::InvalidCast(
                "Array".to_string(),
                "Int".to_string(),
            ))?,
            Val::RuntimeObject(_) => todo!(),
        })
    }

    pub(crate) fn cast_to_float(&self) -> ValResult<f64> {
        Ok(match self {
            Val::Null => 0.,
            Val::Bool(b) => *b as i64 as f64,
            Val::Int(i) => *i as f64,
            Val::Float(f) => *f,
            Val::Char(c) => *c as f64,
            Val::String(PsString(s)) => s.trim().parse::<f64>()?,
            Val::Array(_) => Err(ValError::InvalidCast(
                "Array".to_string(),
                "Float".to_string(),
            ))?,
            Val::RuntimeObject(_) => todo!(),
        })
    }

    pub(super) fn cast_to_string(&self) -> String {
        match self {
            Val::Null => String::new(),
            Val::Bool(b) => String::from(if *b { "True" } else { "False" }),
            Val::Int(i) => i.to_string(),
            Val::Float(f) => f.to_string(),
            Val::Char(c) => char::from_u32(*c).unwrap_or_default().to_string(),
            Val::String(PsString(s)) => s.clone(),
            Val::Array(v) => v
                .iter()
                .map(|val| val.cast_to_string())
                .collect::<Vec<String>>()
                .join(" "),
            Val::RuntimeObject(s) => s.name(),
        }
    }

    pub(super) fn cast_to_join_string(&self) -> String {
        if let Val::Array(_) = self {
            "System.Object[]".to_string()
        } else {
            self.cast_to_string()
        }
    }

    pub(crate) fn cast_to_array(&self) -> Vec<Self> {
        match self {
            Val::Null => vec![],
            Val::Bool(_) | Val::Int(_) | Val::Float(_) | Val::Char(_) | Val::String(_) => {
                vec![self.clone()]
            }
            Val::Array(v) => v.clone(),
            Val::RuntimeObject(_) => todo!(),
        }
    }

    fn repeat(v: &[Val], amount: usize) -> Vec<Val> {
        let mut res = v.to_owned();
        for _ in 1..amount {
            res.append(&mut v.to_owned());
        }
        res
    }

    fn round_bankers(x: f64) -> f64 {
        let rounded = x.trunc();
        let frac = x.fract().abs();

        if frac > 0.5 {
            if x.is_sign_positive() {
                rounded + 1.0
            } else {
                rounded - 1.0
            }
        } else if frac < 0.5 {
            rounded
        } else {
            // exactly halfway: round to even
            if rounded as i64 % 2 == 0 {
                rounded
            } else if x.is_sign_positive() {
                rounded + 1.0
            } else {
                rounded - 1.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut val = Val::Int(4);
        val.add(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::Float(4.1));

        let mut val = Val::String(" 123".into());
        val.add(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::String(" 1230.1".into()));

        let mut val = Val::Char(97);
        val.add(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::String("a0.1".into()));

        let mut val = Val::Int(4);
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::Int(5));

        let mut val = Val::String(" 123".into());
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String(" 1231".into()));

        let mut val = Val::Char(97);
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String("a1".into()));

        let mut val = Val::Char(97);
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String("a1".into()));

        let mut val = Val::String(" 123".into());
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String(" 1231".into()));

        let mut val = Val::Char(97);
        val.add(Val::String("bsef".into())).unwrap();
        assert_eq!(val, Val::String("absef".into()));

        let mut val = Val::Array(vec![Val::Int(7), Val::String(" adsf".into())]);
        val.add(Val::Float(2.3)).unwrap();
        assert_eq!(
            val,
            Val::Array(vec![
                Val::Int(7),
                Val::String(" adsf".into()),
                Val::Float(2.3)
            ])
        );
    }

    #[test]
    fn test_sub() {
        let mut val = Val::Int(4);
        val.sub(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::Float(3.9));

        let mut val = Val::String(" 123".into());
        val.sub(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::Float(122.9));

        let mut val = Val::Char(123);
        val.sub(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::Float(122.9));

        let mut val = Val::Int(4);
        val.sub(Val::Int(1)).unwrap();
        assert_eq!(val, Val::Int(3));

        let mut val = Val::String(" 123".into());
        val.sub(Val::Int(1)).unwrap();
        assert_eq!(val, Val::Int(122));

        let mut val = Val::Char(123);
        val.sub(Val::Int(1)).unwrap();
        assert_eq!(val, Val::Int(122));
    }

    #[test]
    fn test_mul() {
        let mut val = Val::Int(4);
        val.mul(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::Float(0.4));

        let mut val = Val::String(" 123".into());
        val.mul(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::String("".into()));

        let mut val = Val::String(" 123".into());
        val.mul(Val::Float(2.1)).unwrap();
        assert_eq!(val, Val::String(" 123 123".into()));

        // ERROR
        // let mut val = Val::Char(123);
        // val.mul(Val::Float(0.1)).unwrap();
        // assert_eq!(val, Val::Float(122.9));

        let mut val = Val::Int(4);
        val.mul(Val::Int(1)).unwrap();
        assert_eq!(val, Val::Int(4));

        let mut val = Val::String(" 123".into());
        val.mul(Val::Int(2)).unwrap();
        assert_eq!(val, Val::String(" 123 123".into()));

        let mut val = Val::Array(vec![Val::Int(7), Val::String(" adsf".into())]);
        val.mul(Val::Int(2)).unwrap();
        assert_eq!(
            val,
            Val::Array(vec![
                Val::Int(7),
                Val::String(" adsf".into()),
                Val::Int(7),
                Val::String(" adsf".into())
            ])
        );

        let mut val = Val::Array(vec![Val::Int(7), Val::String(" adsf".into())]);
        val.mul(Val::Float(2.3)).unwrap();
        assert_eq!(
            val,
            Val::Array(vec![
                Val::Int(7),
                Val::String(" adsf".into()),
                Val::Int(7),
                Val::String(" adsf".into())
            ])
        );
    }

    #[test]
    fn test_cast_to_bool() {
        assert_eq!(Val::Null.cast_to_bool(), false);
        assert_eq!(Val::Bool(true).cast_to_bool(), true);
        assert_eq!(Val::Bool(false).cast_to_bool(), false);
        assert_eq!(Val::Int(-4).cast_to_bool(), true);
        assert_eq!(Val::Int(0).cast_to_bool(), false);
        assert_eq!(Val::Int(123456).cast_to_bool(), true);
        assert_eq!(Val::Float(0.).cast_to_bool(), false);
        assert_eq!(Val::Float(0.09874).cast_to_bool(), true);
        assert_eq!(Val::Float(-0.09874).cast_to_bool(), true);
        assert_eq!(Val::Char(0).cast_to_bool(), false);
        assert_eq!(Val::Char(97).cast_to_bool(), true);
        assert_eq!(Val::String("a".into()).cast_to_bool(), true);
        assert_eq!(Val::String("  888  a".into()).cast_to_bool(), true);
        assert_eq!(Val::String("".into()).cast_to_bool(), false);
        assert_eq!(Val::Array(vec![]).cast_to_bool(), false);
        assert_eq!(Val::Array(vec![Val::Int(7)]).cast_to_bool(), true);
    }

    #[test]
    fn test_cast_to_char() {
        assert_eq!(Val::Null.cast_to_char().unwrap(), 0);
        assert_eq!(
            Val::Bool(true).cast_to_char().unwrap_err(),
            ValError::InvalidCast("Bool".to_string(), "Char".to_string())
        );
        assert_eq!(
            Val::Bool(false).cast_to_char().unwrap_err(),
            ValError::InvalidCast("Bool".to_string(), "Char".to_string())
        );
        assert_eq!(Val::Int(123456).cast_to_char().unwrap(), 123456);
        assert_eq!(Val::Int(-123456).cast_to_char().unwrap(), 4294843840);
        assert_eq!(
            Val::Float(0.09874).cast_to_char().unwrap_err(),
            ValError::InvalidCast("Float".to_string(), "Char".to_string())
        );
        assert_eq!(
            Val::Float(-0.09874).cast_to_char().unwrap_err(),
            ValError::InvalidCast("Float".to_string(), "Char".to_string())
        );
        assert_eq!(Val::Char(97).cast_to_char().unwrap(), 97);
        assert_eq!(Val::String("a".into()).cast_to_char().unwrap(), 97);
        assert_eq!(
            Val::String("  888  a".into()).cast_to_char().unwrap_err(),
            ValError::InvalidCast(
                "String with len() more than 1".to_string(),
                "Char".to_string()
            )
        );
        assert_eq!(
            Val::Array(vec![Val::Char(7)]).cast_to_char().unwrap_err(),
            ValError::InvalidCast("Array".to_string(), "Char".to_string())
        );
    }

    #[test]
    fn test_cast_to_int() {
        assert_eq!(Val::Null.cast_to_int().unwrap(), 0);
        assert_eq!(Val::Bool(true).cast_to_int().unwrap(), 1);
        assert_eq!(Val::Bool(false).cast_to_int().unwrap(), 0);
        assert_eq!(Val::Int(123456).cast_to_int().unwrap(), 123456);
        assert_eq!(Val::Int(-123456).cast_to_int().unwrap(), -123456);
        assert_eq!(Val::Float(0.09874).cast_to_int().unwrap(), 0);
        assert_eq!(Val::Float(-0.09874).cast_to_int().unwrap(), 0);
        assert_eq!(Val::Char(97).cast_to_int().unwrap(), 97);
        assert_eq!(Val::String("00001".into()).cast_to_int().unwrap(), 1);
        assert_eq!(Val::String("  888  ".into()).cast_to_int().unwrap(), 888);
        assert_eq!(
            Val::String("  888  a".into()).cast_to_int().unwrap_err(),
            ValError::InvalidCast("String".to_string(), "Int".to_string())
        );
        assert_eq!(
            Val::Array(vec![Val::Int(7)]).cast_to_int().unwrap_err(),
            ValError::InvalidCast("Array".to_string(), "Int".to_string())
        );
    }

    #[test]
    fn test_cast_to_float() {
        assert_eq!(Val::Null.cast_to_float().unwrap(), 0.);
        assert_eq!(Val::Bool(true).cast_to_float().unwrap(), 1.);
        assert_eq!(Val::Bool(false).cast_to_float().unwrap(), 0.);
        assert_eq!(Val::Int(123456).cast_to_float().unwrap(), 123456.);
        assert_eq!(Val::Int(-123456).cast_to_float().unwrap(), -123456.);
        assert_eq!(Val::Float(0.09874).cast_to_float().unwrap(), 0.09874);
        assert_eq!(Val::Float(-0.09874).cast_to_float().unwrap(), -0.09874);
        assert_eq!(Val::Char(97).cast_to_float().unwrap(), 97.);
        assert_eq!(Val::String("00001.".into()).cast_to_float().unwrap(), 1.);
        assert_eq!(
            Val::String("00001.12".into()).cast_to_float().unwrap(),
            1.12
        );
        assert_eq!(
            Val::String("  888.123  ".into()).cast_to_float().unwrap(),
            888.123
        );
        assert_eq!(
            Val::String("  888  a".into()).cast_to_float().unwrap_err(),
            ValError::InvalidCast("String".to_string(), "Float".to_string())
        );
        assert_eq!(
            Val::Array(vec![Val::Float(7.)])
                .cast_to_float()
                .unwrap_err(),
            ValError::InvalidCast("Array".to_string(), "Float".to_string())
        );
    }

    #[test]
    fn test_cast_to_string() {
        assert_eq!(Val::Null.cast_to_string(), "".to_string());
        assert_eq!(Val::Bool(true).cast_to_string(), "True".to_string());
        assert_eq!(Val::Bool(false).cast_to_string(), "False".to_string());
        assert_eq!(Val::Int(123456).cast_to_string(), "123456".to_string());
        assert_eq!(Val::Int(-123456).cast_to_string(), "-123456".to_string());
        assert_eq!(Val::Float(1.).cast_to_string(), "1".to_string());
        assert_eq!(Val::Float(0.09874).cast_to_string(), "0.09874".to_string());
        assert_eq!(
            Val::Float(-0.09874).cast_to_string(),
            "-0.09874".to_string()
        );
        assert_eq!(Val::Char(97).cast_to_string(), "a".to_string());
        assert_eq!(Val::Char(9997).cast_to_string(), "\u{270D}".to_string());
        assert_eq!(
            Val::Array(vec![Val::Int(7), Val::Null, Val::String(" adsf".into())]).cast_to_string(),
            "7   adsf".to_string()
        );
    }

    #[test]
    fn test_cast_to_array() {
        assert_eq!(Val::Null.cast_to_array(), vec![]);
        assert_eq!(Val::Bool(true).cast_to_array(), vec![Val::Bool(true)]);
        assert_eq!(
            Val::Float(0.09874).cast_to_array(),
            vec![Val::Float(0.09874)]
        );
        assert_eq!(Val::Char(5).cast_to_array(), vec![Val::Char(5)]);
        assert_eq!(
            Val::String("elo".into()).cast_to_array(),
            vec![Val::String("elo".into())]
        );
        assert_eq!(
            Val::Array(vec![Val::Int(7)]).cast_to_array(),
            vec![Val::Int(7)]
        );
    }
}
