mod method_error;
mod params;
mod ps_string;
mod runtime_object;
mod script_block;
mod system_convert;
mod system_encoding;
mod val_error;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    ops::Neg,
};
mod val_type;
mod class;
pub(super) use val_type::RUNTIME_TYPE_MAP;
pub(super) use class::ClassProperties;
pub(super) use class::ClassType;
pub(crate) use method_error::{MethodError, MethodResult};
pub(crate) use params::Param;
pub(crate) use ps_string::PsString;
use ps_string::str_cmp;
pub(crate) use runtime_object::RuntimeError;
pub(super) use runtime_object::RuntimeObjectTrait;
use runtime_object::{MethodCallType, StaticFnCallType};
pub(crate) use script_block::ScriptBlock;
use smart_default::SmartDefault;
use system_convert::Convert;
pub(crate) use val_error::ValError;
use val_type::RuntimeTypeTrait;
pub(super) use val_type::ValType;
pub type ValResult<T> = core::result::Result<T, ValError>;
use runtime_object::RuntimeResult;

use super::NEWLINE;
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
    HashTable(HashMap<String, Val>),
    RuntimeObject(Box<dyn RuntimeObjectTrait>),
    RuntimeType(Box<dyn RuntimeTypeTrait>),
    ScriptBlock(ScriptBlock),
    ScriptText(String),
    NonDisplayed(Box<Val>),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Val::Null
            | Val::Char(_)
            | Val::Bool(_)
            | Val::Int(_)
            | Val::Float(_)
            | Val::String(_)
            | Val::ScriptBlock(_)
            | Val::ScriptText(_) => self.cast_to_string(),
            Val::HashTable(h) => {
                let mut s = vec![String::from("----                           -----")];
                for (k, v) in h {
                    s.push(format!("{:<30} {}", k, v.cast_to_string()));
                }
                s.join(NEWLINE)
            }
            Val::Array(ar) => ar
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(NEWLINE),
            Val::RuntimeObject(rt) => rt.to_string(),
            Val::RuntimeType(rt) => rt.full_name(),
            Val::NonDisplayed(_) => String::new(),
        };
        write!(f, "{}", str)
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
            (Val::NonDisplayed(box_a), Val::NonDisplayed(box_b)) => *box_a == *box_b,
            (Val::RuntimeType(rt1), Val::RuntimeType(rt2)) => rt1.name() == rt2.name(),
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
            Val::HashTable(a) => Val::HashTable(a.clone()),
            Val::RuntimeObject(ro) => Val::RuntimeObject(ro.clone_rt()),
            Val::RuntimeType(rt) => Val::RuntimeType(rt.clone_rt()),
            Val::ScriptBlock(a) => Val::ScriptBlock(a.clone()),
            Val::ScriptText(a) => Val::ScriptText(a.clone()),
            Val::NonDisplayed(box_val) => Val::NonDisplayed(box_val.clone()),
        }
    }
}
impl Val {
    fn not_defined(v1: &Val, v2: &Val, op: &str) -> ValError {
        ValError::OperationNotDefined(
            op.to_string(),
            v1.ttype().to_string(),
            v2.ttype().to_string(),
        )
    }

    pub fn display(&self) -> String {
        format!("{}", self)
    }

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
            Val::Array(_) => false,
            Val::HashTable(ht1) => {
                if let Val::HashTable(ht2) = val {
                    !ht1.is_empty() && !ht2.is_empty()
                } else {
                    false
                }
            }
            Val::RuntimeObject(s1) => {
                if let Val::RuntimeObject(s2) = val {
                    str_cmp(&s1.name(), &s2.name(), case_insensitive) == std::cmp::Ordering::Equal
                } else {
                    false
                }
            }
            Val::RuntimeType(rt1) => {
                if let Val::RuntimeType(rt2) = val {
                    rt1.name() == rt2.name()
                } else {
                    false
                }
            }
            Val::ScriptBlock(sb1) => {
                if let Val::ScriptBlock(sb2) = val {
                    str_cmp(&sb1.raw_text, &sb2.raw_text, case_insensitive)
                        == std::cmp::Ordering::Equal
                } else {
                    false
                }
            }
            Val::ScriptText(_) => false,
            Val::NonDisplayed(box_val) => box_val.lt(val, case_insensitive)?,
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
            Val::HashTable(_) => false, // HashTables can't be compared with >
            Val::RuntimeObject(_) => todo!(),
            Val::RuntimeType(_) => false, // Add logic if needed
            Val::ScriptBlock(_) => false, // ScriptBlocks can't be compared
            Val::ScriptText(_) => false,
            Val::NonDisplayed(box_val) => box_val.gt(val, case_insensitive)?,
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
            Val::HashTable(_) => false, // HashTables can't be compared with <
            Val::RuntimeObject(_) => todo!(),
            Val::RuntimeType(_) => false, // Add logic if needed
            Val::ScriptBlock(_) => false, // ScriptBlocks can't be compared
            Val::ScriptText(_) => false,
            Val::NonDisplayed(box_val) => box_val.lt(val, case_insensitive)?,
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
            Val::Array(_) => ValType::Array(None),
            Val::HashTable(_) => ValType::HashTable,
            Val::ScriptBlock(_) => ValType::ScriptBlock,
            Val::ScriptText(_) => ValType::ScriptText,
            Val::RuntimeObject(rt) => ValType::RuntimeObject(rt.to_string()),
            Val::RuntimeType(_) => ValType::TypeInfo,
            Val::NonDisplayed(box_val) => box_val.ttype(),
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
            Val::Array(arr) => {
                if let Val::Array(val_arr) = val {
                    arr.extend(val_arr);
                } else {
                    arr.push(val);
                }
            }
            Val::HashTable(ht) => {
                if val.ttype() != ValType::HashTable {
                    return Err(ValError::OperationNotDefined(
                        "add".to_string(),
                        self.ttype().to_string(),
                        val.ttype().to_string(),
                    ));
                } else {
                    ht.extend(val.cast_to_hashtable()?);
                }
            }
            Val::NonDisplayed(box_val) => box_val.add(val)?,
            _ => {
                return Err(ValError::OperationNotDefined(
                    "add".to_string(),
                    self.ttype().to_string(),
                    val.ttype().to_string(),
                ));
            }
        }
        Ok(())
    }

    fn inc_or_dec_operation(&mut self, amount: i64, op: String) -> ValResult<()> {
        match self {
            Val::Null => *self = Val::Int(amount),
            Val::Int(i) => *self = Val::Int(*i + amount),
            Val::Float(f) => *self = Val::Float(*f + amount as f64),
            Val::NonDisplayed(box_val) => box_val.inc_or_dec_operation(amount, op)?,
            _ => {
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
        if let ValType::RuntimeObject(_) = self.ttype() {
            Err(Self::not_defined(self, &val, "-"))?
        }

        if let ValType::RuntimeObject(_) = val.ttype() {
            Err(Self::not_defined(self, &val, "-"))?
        }

        if let ValType::TypeInfo = self.ttype() {
            Err(Self::not_defined(self, &val, "-"))?
        }

        if let ValType::TypeInfo = val.ttype() {
            Err(Self::not_defined(self, &val, "-"))?
        }

        if let ValType::Array(_) = self.ttype() {
            Err(Self::not_defined(self, &val, "-"))?
        }

        if let ValType::Array(_) = val.ttype() {
            Err(Self::not_defined(self, &val, "-"))?
        }

        if self.ttype() == ValType::ScriptBlock || val.ttype() == ValType::ScriptBlock {
            Err(Self::not_defined(self, &val, "-"))?
        }

        if self.ttype() == ValType::Float || val.ttype() == ValType::Float {
            *self = Val::Float(self.cast_to_float()? - val.cast_to_float()?);
        } else {
            *self = Val::Int(self.cast_to_int()? - val.cast_to_int()?);
        }

        Ok(())
    }

    pub fn mul(&mut self, val: Val) -> ValResult<()> {
        let res = match self {
            Val::Null => Ok(self.clone()),
            Val::Bool(_) => Err(Self::not_defined(self, &val, "*")),
            Val::Int(_) | Val::Float(_) => {
                if self.ttype() == ValType::Float || val.ttype() == ValType::Float {
                    Ok(Val::Float(self.cast_to_float()? * val.cast_to_float()?))
                } else {
                    Ok(Val::Int(self.cast_to_int()? * val.cast_to_int()?))
                }
            }
            Val::Char(_) => Err(ValError::OperationNotDefined(
                "*".to_string(),
                "Char".to_string(),
                val.ttype().to_string(),
            ))?,
            Val::String(PsString(s)) => {
                let repeat_count = val.cast_to_int()?;
                if repeat_count < 0 {
                    Err(ValError::ArgumentOutOfRange("*".to_string(), repeat_count))?
                }
                Ok(Val::String(PsString(s.repeat(repeat_count as usize))))
            }
            Val::Array(v) => {
                let repeat_count = val.cast_to_int()?;
                if repeat_count < 0 {
                    Err(ValError::ArgumentOutOfRange("*".to_string(), repeat_count))?
                }
                Ok(Val::Array(Self::repeat(v, repeat_count as usize)))
            }
            _ => Err(ValError::OperationNotDefined(
                "*".to_string(),
                self.ttype().to_string(),
                val.ttype().to_string(),
            )),
        };
        *self = res?;
        Ok(())
    }

    pub fn div(&mut self, val: Val) -> ValResult<()> {
        if let Val::Array(_) = self {
            Err(Self::not_defined(self, &val, "/"))?
        }

        if let Val::Array(_) = &val {
            Err(Self::not_defined(self, &val, "/"))?
        }

        // check dividing by zero
        if let Ok(v) = val.cast_to_float()
            && v == 0.
        {
            Err(ValError::DividingByZero)?
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
            _ => Err(ValError::OperationNotDefined(
                "/".to_string(),
                self.ttype().to_string(),
                val.ttype().to_string(),
            ))?,
        };
        Ok(())
    }

    pub fn modulo(&mut self, val: Val) -> ValResult<()> {
        if let Val::Array(_) = self {
            Err(Self::not_defined(self, &val, "%"))?
        }

        if let Val::Array(_) = &val {
            Err(Self::not_defined(self, &val, "%"))?
        }

        // check dividing by zero
        if let Ok(v) = val.cast_to_float()
            && v == 0.
        {
            Err(ValError::DividingByZero)?
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
            _ => Err(ValError::OperationNotDefined(
                "%".to_string(),
                self.ttype().to_string(),
                val.ttype().to_string(),
            ))?,
        };
        Ok(())
    }

    pub fn neg(&mut self) -> ValResult<()> {
        match self {
            Val::Float(f) => *f = f.neg(),
            Val::Null | Val::Bool(_) | Val::Int(_) | Val::Char(_) | Val::String(_) => {
                *self = Val::Int(self.cast_to_int()?.neg())
            }
            Val::NonDisplayed(box_val) => box_val.neg()?,
            _ => Err(ValError::OperationNotDefined(
                "-".to_string(),
                self.ttype().to_string(),
                self.ttype().to_string(),
            ))?,
        }
        Ok(())
    }

    pub(crate) fn cast(&self, runtime_type: &Val) -> ValResult<Self> {
        let Val::RuntimeType(rt) = runtime_type else {
            return Err(ValError::InvalidCast(
                self.ttype().to_string(),
                "RuntimeType".to_string(),
            ))?;
        };
        self.cast_from_type(&rt.type_definition())
    }

    pub(crate) fn cast_from_type(&self, ttype: &ValType) -> ValResult<Self> {
        Ok(match ttype {
            ValType::Null => Err(ValError::UnknownType("Null".to_string()))?,
            ValType::Bool => Val::Bool(self.cast_to_bool()),
            ValType::Int => Val::Int(self.cast_to_int()?),
            ValType::Float => Val::Float(self.cast_to_float()?),
            ValType::Char => Val::Char(self.cast_to_char()?),
            ValType::String => Val::String(PsString(self.cast_to_string())),
            ValType::Array(ttype) => Val::Array(self.cast_to_typed_array(ttype.clone())?),
            ValType::HashTable => Val::HashTable(self.cast_to_hashtable()?),
            ValType::ScriptBlock => Val::ScriptBlock(self.cast_to_scriptblock()?),
            ValType::ScriptText => Val::ScriptText(self.cast_to_script()),
            ValType::RuntimeObject(_) => Err(ValError::InvalidCast(
                self.ttype().to_string(),
                "RuntimeObject".to_string(),
            ))?,
            ValType::TypeInfo => Err(ValError::InvalidCast(
                self.ttype().to_string(),
                "TypeInfo".to_string(),
            ))?,
            ValType::Switch => Err(ValError::InvalidCast(
                self.ttype().to_string(),
                "Switch".to_string(),
            ))?,
        })
    }

    // pub(crate) fn init(ttype: ValType) -> ValResult<Self> {
    //     Ok(match ttype {
    //         ValType::Null => Err(ValError::UnknownType("Null".to_string()))?,
    //         ValType::Bool => Val::Bool(false),
    //         ValType::Int => Val::Int(0),
    //         ValType::Float => Val::Float(0.),
    //         ValType::Char => Val::Char(0),
    //         ValType::String => Val::String(PsString::default()),
    //         ValType::Array(_) => Val::Array(Default::default()),
    //         ValType::HashTable => Val::HashTable(HashMap::new()),
    //         ValType::ScriptBlock => Val::ScriptBlock(ScriptBlock::default()),
    //         ValType::ScriptText => Val::ScriptText("".to_string()),
    //         //ValType::RuntimeObject(s) =>
    // Val::RuntimeObject(ValType::runtime_object_from_name(s.to_string().as_str()).
    // unwrap_or_default()),         ValType::RuntimeObject(_) =>
    // Err(ValError::UnknownType("Can't init RuntimeObject".into()))?,
    //         ValType::TypeInfo => Err(ValError::UnknownType("Can't init
    // TypeInfo".into()))?,         ValType::Switch =>
    // Err(ValError::UnknownType("Can't init Switch".into()))?,     })
    // }

    pub(crate) fn cast_to_bool(&self) -> bool {
        match self {
            Val::Null => false,
            Val::Bool(b) => *b,
            Val::Char(c) => *c != 0,
            Val::Int(i) => *i != 0,
            Val::Float(f) => *f != 0.,
            Val::String(PsString(s)) => !s.is_empty(),
            Val::Array(v) => !v.is_empty(),
            Val::HashTable(h) => !h.is_empty(),
            Val::RuntimeObject(rt) => !rt.name().is_empty(),
            Val::RuntimeType(_rt) => true,
            Val::ScriptBlock(_) => true,
            Val::ScriptText(st) => !st.is_empty(),
            Val::NonDisplayed(box_val) => box_val.cast_to_bool(),
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
            Val::HashTable(_) => Err(ValError::InvalidCast(
                "HashTable".to_string(),
                "Char".to_string(),
            ))?,
            Val::RuntimeObject(_) => {
                Err(ValError::InvalidCast(self.to_string(), "Char".to_string()))?
            }
            Val::RuntimeType(_) => {
                Err(ValError::InvalidCast(self.to_string(), "Char".to_string()))?
            }
            Val::ScriptBlock(_) => Err(ValError::InvalidCast(
                "ScriptBlock".to_string(),
                "Char".to_string(),
            ))?,
            Val::ScriptText(_) => Err(ValError::InvalidCast(
                "ScriptText".to_string(),
                "Char".to_string(),
            ))?,
            Val::NonDisplayed(box_val) => box_val.cast_to_char()?,
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
                    s.trim()
                        .parse::<i64>()
                        .map_err(|_| ValError::InvalidCast(format!("\"{s}\""), "Int".to_string()))?
                }
            }
            Val::Array(_) => Err(ValError::InvalidCast(
                "Array".to_string(),
                "Int".to_string(),
            ))?,
            Val::HashTable(_) => Err(ValError::InvalidCast(
                "HashTable".to_string(),
                "Int".to_string(),
            ))?,
            Val::RuntimeObject(_) => {
                Err(ValError::InvalidCast(self.to_string(), "Int".to_string()))?
            }
            Val::RuntimeType(_) => Err(ValError::InvalidCast(self.to_string(), "Int".to_string()))?,
            Val::ScriptBlock(_) => Err(ValError::InvalidCast(
                "ScriptBlock".to_string(),
                "Int".to_string(),
            ))?,
            Val::ScriptText(_) => Err(ValError::InvalidCast(
                "ScriptText".to_string(),
                "Int".to_string(),
            ))?,
            Val::NonDisplayed(box_val) => box_val.cast_to_int()?,
        })
    }

    pub(crate) fn cast_to_float(&self) -> ValResult<f64> {
        Ok(match self {
            Val::Null => 0.,
            Val::Bool(b) => *b as i64 as f64,
            Val::Int(i) => *i as f64,
            Val::Float(f) => *f,
            Val::Char(c) => *c as f64,
            Val::String(PsString(s)) => s
                .trim()
                .parse::<f64>()
                .map_err(|_| ValError::InvalidCast(format!("\"{s}\""), "Float".to_string()))?,
            Val::Array(_) => Err(ValError::InvalidCast(
                "Array".to_string(),
                "Float".to_string(),
            ))?,
            Val::HashTable(_) => Err(ValError::InvalidCast(
                "HashTable".to_string(),
                "Float".to_string(),
            ))?,
            Val::RuntimeObject(_) => {
                Err(ValError::InvalidCast(self.to_string(), "Float".to_string()))?
            }
            Val::RuntimeType(_) => Err(ValError::InvalidCast(
                self.to_string(),
                "InFloatt".to_string(),
            ))?,
            Val::ScriptBlock(_) => Err(ValError::InvalidCast(
                "ScriptBlock".to_string(),
                "Float".to_string(),
            ))?,
            Val::ScriptText(_) => Err(ValError::InvalidCast(
                "ScriptText".to_string(),
                "Float".to_string(),
            ))?,
            Val::NonDisplayed(box_val) => box_val.cast_to_float()?,
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
            Val::HashTable(_) => "System.Collections.Hashtable".to_string(),
            Val::RuntimeObject(s) => s.to_string(),
            Val::RuntimeType(_) => self.to_string(),
            Val::ScriptBlock(sb) => sb.to_string(),
            Val::ScriptText(st) => st.clone(),
            Val::NonDisplayed(box_val) => box_val.cast_to_string(),
        }
    }

    pub(super) fn cast_to_join_string(&self) -> String {
        if let Val::Array(_) = self {
            "System.Object[]".to_string()
        } else {
            self.cast_to_string()
        }
    }

    pub(crate) fn cast_to_typed_array(&self, ttype: Option<Box<ValType>>) -> ValResult<Vec<Self>> {
        if let Some(ttype) = &ttype
            && **ttype == ValType::String
        {
            return Ok(self
                .to_string()
                .split_ascii_whitespace()
                .map(|s| Val::String(s.into()))
                .collect());
        }
        let mut arr = match self {
            Val::Null => vec![],
            Val::Bool(_) | Val::Int(_) | Val::Float(_) | Val::Char(_) | Val::String(_) => {
                vec![self.clone()]
            }
            Val::Array(v) => v.clone(),
            Val::HashTable(_) => vec![self.clone()],
            Val::RuntimeObject(a) => vec![Val::String(a.name().into())],
            Val::ScriptBlock(sb) => vec![Val::String(sb.to_string().into())],
            Val::ScriptText(s) => vec![Val::String(s.clone().into())],
            Val::NonDisplayed(s) => s.cast_to_typed_array(ttype.clone())?,
            _ => Err(ValError::InvalidCast(
                self.ttype().to_string(),
                "Array".to_string(),
            ))?,
        };
        if let Some(ttype) = ttype {
            for elem in arr.iter_mut() {
                *elem = elem.cast_from_type(&ttype)?;
            }
        }
        Ok(arr)
    }

    pub(crate) fn cast_to_array(&self) -> Vec<Self> {
        self.cast_to_typed_array(None).unwrap_or_default()
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

    pub(crate) fn cast_to_hashtable(&self) -> ValResult<HashMap<String, Val>> {
        if let Val::HashTable(h) = self {
            Ok(h.clone())
        } else {
            Err(ValError::InvalidCast(
                self.ttype().to_string(),
                "HashTable".to_string(),
            ))
        }
    }

    pub(crate) fn cast_to_scriptblock(&self) -> ValResult<ScriptBlock> {
        if let Val::ScriptBlock(sb) = self {
            Ok(sb.clone())
        } else {
            Err(ValError::InvalidCast(
                self.ttype().to_string(),
                "ScriptBlock".to_string(),
            ))
        }
    }

    pub fn get_index_ref(&mut self, index: Val) -> ValResult<&mut Val> {
        let self_string = self.to_string();
        match self {
            Val::Null => Err(ValError::IndexedNullArray)?,
            Val::Array(v) => {
                let i = index.cast_to_int()? as usize;
                if v.len() > i {
                    Ok(&mut v[i])
                } else {
                    Err(RuntimeError::IndexOutOfBounds(self_string, i).into())
                }
            }
            Val::HashTable(v) => v
                .get_mut(&index.cast_to_string().to_ascii_lowercase())
                .ok_or(RuntimeError::MemberNotFound(index.cast_to_string()).into()),
            _ => {
                if let Ok(i) = index.cast_to_int() {
                    if i == 0 {
                        Ok(self)
                    } else {
                        Err(RuntimeError::IndexOutOfBounds(self_string, i as usize).into())
                    }
                } else {
                    let member_name = index.cast_to_string();
                    Err(RuntimeError::MemberNotFound(member_name).into())
                }
            }
        }
    }

    pub fn get_index(&self, index: Val) -> ValResult<Val> {
        let self_string = self.to_string();
        match self {
            Val::Null => Err(ValError::IndexedNullArray)?,
            Val::Array(v) => {
                let i = index.cast_to_int()? as usize;
                if v.len() > i {
                    Ok(v[i].clone())
                } else {
                    Err(RuntimeError::IndexOutOfBounds(self_string, i).into())
                }
            }
            Val::String(PsString(s)) => {
                let i = index.cast_to_int()? as usize;

                let Some(c) = s.chars().nth(i) else {
                    return Err(RuntimeError::IndexOutOfBounds(self_string, i).into());
                };
                Ok(Val::Char(c as u32))
            }
            Val::HashTable(v) => v
                .get(&index.cast_to_string().to_ascii_lowercase())
                .cloned()
                .ok_or(RuntimeError::MemberNotFound(index.cast_to_string()).into()),
            _ => {
                if let Ok(i) = index.cast_to_int() {
                    if i == 0 {
                        Ok(self.clone())
                    } else {
                        Err(RuntimeError::IndexOutOfBounds(self_string, i as usize).into())
                    }
                } else {
                    let member_name = index.cast_to_string();
                    Err(RuntimeError::MemberNotFound(member_name).into())
                }
            }
        }
    }

    pub fn flatten(&self) -> Vec<Self> {
        match self {
            Val::Array(v) => {
                let mut res = vec![];
                for item in v {
                    res.append(&mut item.flatten());
                }
                res
            }
            Val::HashTable(_) => vec![Val::String(self.cast_to_string().into())],
            _ => self.cast_to_array(),
        }
    }

    pub(super) fn cast_to_script(&self) -> String {
        log::trace!("cast_to_script {:?}", self);
        match self {
            Val::Null => "$null".to_string(),
            Val::Bool(b) => String::from(if *b { "$true" } else { "$false" }),
            Val::Int(i) => i.to_string(),
            Val::Float(f) => f.to_string(),
            Val::Char(c) => format!("'{}'", char::from_u32(*c).unwrap_or_default()),
            Val::String(PsString(s)) => format!("\"{}\"", s),
            Val::Array(v) => {
                let inner = v
                    .iter()
                    .map(|val| val.cast_to_script())
                    .collect::<Vec<String>>()
                    .join(",");
                format!("@({})", inner)
            }
            Val::HashTable(h) => {
                let tree_map = BTreeMap::from_iter(h.clone());
                let inner = tree_map
                    .iter()
                    .map(|(k, v)| format!("\t{} = {}", k, v.cast_to_script()))
                    .collect::<Vec<String>>()
                    .join(NEWLINE);
                format!("@{{{NEWLINE}{}{NEWLINE}}}", inner)
            }
            Val::RuntimeObject(s) => s.to_string(),
            Val::RuntimeType(s) => format!("[{}]", s.full_name()),
            Val::ScriptBlock(sb) => format!("{{{}}}", sb),
            Val::ScriptText(st) => st.clone(),
            Val::NonDisplayed(box_val) => (*box_val).cast_to_script(),
        }
    }
}

impl From<&str> for Val {
    fn from(value: &str) -> Self {
        Self::String(PsString(value.into()))
    }
}

impl From<String> for Val {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&String> for Val {
    fn from(value: &String) -> Self {
        value.as_str().into()
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
            ValError::InvalidCast("\"  888  a\"".to_string(), "Int".to_string())
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
            ValError::InvalidCast("\"  888  a\"".to_string(), "Float".to_string())
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

        assert_eq!(
            Val::Array(vec![Val::Int(7)])
                .cast_to_typed_array(Some(Box::new(ValType::String)))
                .unwrap(),
            vec![Val::String("7".into())]
        );
    }
}
