use std::{num::ParseFloatError, sync::LazyLock};
use std::cmp::Ordering;
use thiserror_no_std::Error;

// very strange. En-us culture has different ordering than default. A (ascii 65)
// is greater than a(ascii 97 need to Collator object to perform string
// comparison
#[cfg(feature = "en-us")]
const COLLATOR: LazyLock<icu::collator::Collator> = LazyLock::new(|| {
    icu::collator::Collator::try_new(
        &icu::locid::locale!("en-US").into(),
        icu::collator::CollatorOptions::new(),
    )
    .unwrap()
});

fn str_cmp(s1: &str, s2: &str, case_insensitive: bool) -> Ordering {
    if case_insensitive {
        s1.to_ascii_lowercase().cmp(&s2.to_ascii_lowercase())
    } else {
        if cfg!(feature = "en-us") {
            COLLATOR.compare(s1, s2)
        } else {
            s1.cmp(s2)
        }
    }
}


#[derive(Error, Debug, PartialEq)]
pub enum ValError {
    #[error("Cannot convert value \"{0}\" to type \"{1}\"")]
    InvalidCast(String, String),

    #[error("Unknown type \"{0}\"")]
    UnknownType(String),

    #[error("Operation \"{0}\" is not defined for types \"{1}\" op \"{2}\"")]
    OperationNotDefined(String, String, String),

    #[error("Can't divide by zero")]
    DividingByZero,
}

impl From<ParseFloatError> for ValError {
    fn from(_value: ParseFloatError) -> Self {
        Self::InvalidCast("String".to_string(), "Int".to_string())
    }
}
use smart_default::SmartDefault;
type ValResult<T> = core::result::Result<T, ValError>;

#[derive(PartialEq, Debug)]
pub enum ValType {
    Null,
    Bool,
    Int,
    Float,
    Char,
    String,
}

impl std::fmt::Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ValType {
    pub(crate) fn cast(s: &str) -> ValResult<Self> {
        let s = s.to_ascii_lowercase();
        let t = match s.as_str() {
            "char" | "byte" => Self::Char,
            "bool" => Self::Bool,
            "int" | "long" | "decimal" => Self::Int,
            "float" | "double" => Self::Float,
            "string" => Self::String,
            _ => Err(ValError::UnknownType(s))?,
        };
        Ok(t)
    }
}

#[derive(Clone, Debug, SmartDefault, PartialEq)]
pub enum Val {
    #[default]
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Char(u32),
    String(String),
}

impl Val {
    pub fn eq(&self, val: Val, case_insensitive: bool) -> ValResult<bool> {
        Ok(match self {
            Val::Null => val.ttype() == ValType::Null,
            Val::Bool(b) => *b == val.cast_to_bool()?,
            Val::Char(c) => *c == val.cast_to_char()?,
            Val::Int(i) => *i == val.cast_to_int()?,
            Val::Float(f) => *f == val.cast_to_float()?,
            Val::String(s1) => {
                let s2 = val.cast_to_string();
                str_cmp(s1, &s2, case_insensitive) == std::cmp::Ordering::Equal
            }
        })
    }

    pub fn gt(&self, val: Val, case_insensitive: bool) -> ValResult<bool> {
        Ok(match self {
            Val::Null => false,
            Val::Bool(b) => *b > val.cast_to_bool()?,
            Val::Char(c) => *c > val.cast_to_char()?,
            Val::Int(i) => *i > val.cast_to_int()?,
            Val::Float(f) => *f > val.cast_to_float()?,
            Val::String(s1) => {
                let s2 = val.cast_to_string();
                str_cmp(s1, &s2, case_insensitive) == std::cmp::Ordering::Greater
            }
        })
    }

    pub fn lt(&self, val: Val, case_insensitive: bool) -> ValResult<bool> {
        Ok(match self {
            Val::Null => false,
            Val::Bool(b) => *b < val.cast_to_bool()?,
            Val::Char(c) => *c < val.cast_to_char()?,
            Val::Int(i) => *i < val.cast_to_int()?,
            Val::Float(f) => *f < val.cast_to_float()?,
            Val::String(s1) => {
                let s2 = val.cast_to_string();
                str_cmp(s1, &s2, case_insensitive) == std::cmp::Ordering::Less
            }
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
        }
    }

    pub fn add(&mut self, val: Val) -> ValResult<()> {
        let res = match self {
            Val::Null => *self = val,
            Val::Bool(_) | Val::Int(_) | Val::Float(_) => {
                *self = if val.ttype() == ValType::Float {
                    Val::Float(self.cast_to_float()? + val.cast_to_float()?)
                } else {
                    Val::Int(self.cast_to_int()? + val.cast_to_int()?)
                };
            }
            Val::Char(_) | Val::String(_) => {
                *self = Val::String(self.cast_to_string() + val.cast_to_string().as_str())
            }
        };
        Ok(res)
    }

    pub fn sub(&mut self, val: Val) -> ValResult<()> {
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
            Val::String(_) => {
                Val::String(self.cast_to_string().repeat(val.cast_to_int()? as usize))
            }
        };
        Ok(())
    }

    pub fn div(&mut self, val: Val) -> ValResult<()> {
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
        };
        Ok(())
    }

    pub fn modulo(&mut self, val: Val) -> ValResult<()> {
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
        };
        Ok(())
    }

    pub(crate) fn cast(&mut self, ttype: ValType) -> ValResult<Self> {
        Ok(match ttype {
            ValType::Null => Err(ValError::UnknownType("Null".to_string()))?,
            ValType::Bool => Val::Bool(self.cast_to_bool()?),
            ValType::Int => Val::Int(self.cast_to_int()?),
            ValType::Float => Val::Float(self.cast_to_float()?),
            ValType::Char => Val::Char(self.cast_to_char()?),
            ValType::String => Val::String(self.cast_to_string()),
        })
    }

    fn cast_to_bool(&self) -> ValResult<bool> {
        let res = match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Int(_) | Val::Float(_) | Val::Char(_) => self.cast_to_float()? != 0.,
            Val::String(s) => !s.is_empty(),
        };
        Ok(res)
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
            Val::String(s) => {
                if s.len() == 1 {
                    s.chars().next().unwrap_or_default() as u32
                } else {
                    Err(ValError::InvalidCast(
                        "String with len() more than 1".to_string(),
                        "Char".to_string(),
                    ))?
                }
            }
        };
        Ok(res)
    }

    fn cast_to_int(&self) -> ValResult<i64> {
        Ok(self.cast_to_float()? as i64)
    }

    fn cast_to_float(&self) -> ValResult<f64> {
        Ok(match self {
            Val::Null => 0.,
            Val::Bool(b) => *b as i64 as f64,
            Val::Int(i) => *i as f64,
            Val::Float(f) => *f as f64,
            Val::Char(c) => *c as f64,
            Val::String(s) => s.trim().parse::<f64>()?,
        })
    }

    pub(super) fn cast_to_string(&self) -> String {
        match self {
            Val::Null => String::new(),
            Val::Bool(b) => String::from(if *b { "True" } else { "False" }),
            Val::Int(i) => i.to_string(),
            Val::Float(f) => f.to_string(),
            Val::Char(c) => char::from_u32(*c).unwrap_or_default().to_string(),
            Val::String(s) => s.clone(),
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

        let mut val = Val::String(" 123".to_string());
        val.add(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::String(" 1230.1".to_string()));

        let mut val = Val::Char(97);
        val.add(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::String("a0.1".to_string()));

        let mut val = Val::Int(4);
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::Int(5));

        let mut val = Val::String(" 123".to_string());
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String(" 1231".to_string()));

        let mut val = Val::Char(97);
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String("a1".to_string()));

        let mut val = Val::Char(97);
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String("a1".to_string()));

        let mut val = Val::String(" 123".to_string());
        val.add(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String(" 1231".to_string()));

        let mut val = Val::Char(97);
        val.add(Val::String("bsef".to_string())).unwrap();
        assert_eq!(val, Val::String("absef".to_string()));
    }

    #[test]
    fn test_sub() {
        let mut val = Val::Int(4);
        val.sub(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::Float(3.9));

        let mut val = Val::String(" 123".to_string());
        val.sub(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::Float(122.9));

        let mut val = Val::Char(123);
        val.sub(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::Float(122.9));

        let mut val = Val::Int(4);
        val.sub(Val::Int(1)).unwrap();
        assert_eq!(val, Val::Int(3));

        let mut val = Val::String(" 123".to_string());
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

        let mut val = Val::String(" 123".to_string());
        val.mul(Val::Float(0.1)).unwrap();
        assert_eq!(val, Val::String("".to_string()));

        let mut val = Val::String(" 123".to_string());
        val.mul(Val::Float(2.1)).unwrap();
        assert_eq!(val, Val::String(" 123 123".to_string()));

        // ERROR
        // let mut val = Val::Char(123);
        // val.mul(Val::Float(0.1)).unwrap();
        // assert_eq!(val, Val::Float(122.9));

        let mut val = Val::Int(4);
        val.mul(Val::Int(1)).unwrap();
        assert_eq!(val, Val::Int(4));

        let mut val = Val::String(" 123".to_string());
        val.mul(Val::Int(1)).unwrap();
        assert_eq!(val, Val::String(" 123".to_string()));
    }

    #[test]
    fn test_cast_to_bool() {
        assert_eq!(Val::Null.cast_to_bool().unwrap(), false);
        assert_eq!(Val::Bool(true).cast_to_bool().unwrap(), true);
        assert_eq!(Val::Bool(false).cast_to_bool().unwrap(), false);
        assert_eq!(Val::Int(-4).cast_to_bool().unwrap(), true);
        assert_eq!(Val::Int(0).cast_to_bool().unwrap(), false);
        assert_eq!(Val::Int(123456).cast_to_bool().unwrap(), true);
        assert_eq!(Val::Float(0.).cast_to_bool().unwrap(), false);
        assert_eq!(Val::Float(0.09874).cast_to_bool().unwrap(), true);
        assert_eq!(Val::Float(-0.09874).cast_to_bool().unwrap(), true);
        assert_eq!(Val::Char(0).cast_to_bool().unwrap(), false);
        assert_eq!(Val::Char(97).cast_to_bool().unwrap(), true);
        assert_eq!(Val::String("a".to_string()).cast_to_bool().unwrap(), true);
        assert_eq!(
            Val::String("  888  a".to_string()).cast_to_bool().unwrap(),
            true
        );
        assert_eq!(Val::String("".to_string()).cast_to_bool().unwrap(), false);
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
        assert_eq!(Val::String("a".to_string()).cast_to_char().unwrap(), 97);
        assert_eq!(
            Val::String("  888  a".to_string())
                .cast_to_char()
                .unwrap_err(),
            ValError::InvalidCast(
                "String with len() more than 1".to_string(),
                "Char".to_string()
            )
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
        assert_eq!(Val::String("00001".to_string()).cast_to_int().unwrap(), 1);
        assert_eq!(
            Val::String("  888  ".to_string()).cast_to_int().unwrap(),
            888
        );
        assert_eq!(
            Val::String("  888  a".to_string())
                .cast_to_int()
                .unwrap_err(),
            ValError::InvalidCast("String".to_string(), "Int".to_string())
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
        assert_eq!(
            Val::String("00001.".to_string()).cast_to_float().unwrap(),
            1.
        );
        assert_eq!(
            Val::String("00001.12".to_string()).cast_to_float().unwrap(),
            1.12
        );
        assert_eq!(
            Val::String("  888.123  ".to_string())
                .cast_to_float()
                .unwrap(),
            888.123
        );
        assert_eq!(
            Val::String("  888  a".to_string())
                .cast_to_float()
                .unwrap_err(),
            ValError::InvalidCast("String".to_string(), "Int".to_string())
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
    }
}
