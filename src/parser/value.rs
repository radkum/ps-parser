use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum ValError {
    #[error("Cannot convert value \"{0}\" to type \"{1}\"")]
    InvalidCast(String, String),

    #[error("Unknown type \"{0}\"")]
    UnknownType(String),
}

use smart_default::SmartDefault;
type ValResult<T> = core::result::Result<T, ValError>;

#[derive(PartialEq)]
pub enum ValType {
    Null,
    Bool,
    Int,
    Float,
    Char,
    String,
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

#[derive(Clone, Debug, SmartDefault)]
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
    pub fn eq(&self, val: Val, case_sensitive: bool) -> ValResult<bool> {
        Ok(match self {
            Val::Null => val.ttype() == ValType::Null,
            Val::Bool(b) => *b == val.cast_to_bool()?,
            Val::Char(c) => *c == val.cast_to_char()?,
            Val::Int(i) => *i == val.cast_to_int()?,
            Val::Float(f) => *f == val.cast_to_float()?,
            Val::String(s) => {
                let s2 = val.cast_to_string()?;
                if case_sensitive {
                    s.eq_ignore_ascii_case(s2.as_str())
                } else {
                    s == &s2
                }
                
            },
        })
    }

    fn ttype(&self) -> ValType {
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
            Val::Char(_) | Val::String(_) => *self = Val::String(self.cast_to_string()? + val.cast_to_string()?.as_str()),
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

    pub(crate) fn cast(&mut self, ttype: ValType) -> ValResult<Self> {
        Ok(match ttype {
            ValType::Null => Err(ValError::UnknownType("Null".to_string()))?,
            ValType::Bool => Val::Bool(self.cast_to_bool()?),
            ValType::Int => Val::Int(self.cast_to_int()?),
            ValType::Float => Val::Float(self.cast_to_float()?),
            ValType::Char => Val::Char(self.cast_to_char()?),
            ValType::String => Val::String(self.cast_to_string()?),
        })
    }

    fn cast_to_bool(&self) -> ValResult<bool> {
        let res = match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Int(_) | Val::Float(_) | Val::Char(_) => self.cast_to_int()? != 0,
            Val::String(s) => !s.is_empty(),
        };
        Ok(res)
    }

    fn cast_to_char(&self) -> ValResult<u32> {
        let res = match self {
            Val::Null | Val::Int(_) | Val::Char(_) => self.cast_to_int()? as u32,
            Val::Bool(_) => Err(ValError::InvalidCast("bool".to_string(), "char".to_string()))?,
            Val::Float(_) => Err(ValError::InvalidCast("Float".to_string(), "char".to_string()))?,
            Val::String(s) => {
                if s.len() == 1 { 
                    s.chars().next().unwrap_or_default() as u32
                } else {
                    Err(ValError::InvalidCast("String with len() more than 1".to_string(), "char".to_string()))?
                }
            }
        };
        Ok(res)
    }

    fn cast_to_int(&self) -> ValResult<i64> {
        let res = match self {
            Val::Null => 0,
            Val::Bool(b) => *b as i64,
            Val::Int(i) => *i,
            Val::Float(f) => *f as i64,
            Val::Char(c) => *c as i64,
            Val::String(s) => Err(ValError::InvalidCast(s.clone(), "Int".to_string()))?,
        };
        Ok(res)
    }

    fn cast_to_float(&self) -> ValResult<f64> {
        Ok(if let Val::Float(f) = self {
            *f
        } else {
            self.cast_to_int()? as f64
        })
    }

    fn cast_to_string(&self) -> ValResult<String> {
        let res = match self {
            Val::Null => String::new(),
            Val::Bool(b) => String::from(if *b { "True"} else {"False"}),
            Val::Int(i) => i.to_string(),
            Val::Float(f) => f.to_string(),
            Val::Char(c) => char::from_u32(*c).unwrap_or_default().to_string(),
            Val::String(s) => s.clone(),
        };
        Ok(res)
    }
}
