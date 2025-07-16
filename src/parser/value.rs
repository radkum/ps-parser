pub enum ValType {
    Bool,
    Int,
    Char,
    String,
}

impl ValType {
    pub(crate) fn cast(s: &str) -> Self {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "char" | "byte" => Self::Char,
            "bool" => Self::Bool,
            "int" => Self::Int,
            "string" => Self::String,
            _ => todo!(),
        }
    }

    pub(crate) fn init(&mut self, val: Val) -> Val {
        match self {
            Self::Bool => todo!(),
            Self::Int => todo!(),
            Self::Char => Val::Char(val.into_char()),
            Self::String => todo!(),
        }
    }
}

use smart_default::SmartDefault;
#[derive(Clone, Debug, SmartDefault)]
pub enum Val {
    #[default]
    Null,
    Int(i64),
    Char(u32),
    String(String),
}

impl Val {
    fn into_char(&self) -> u32 {
        match self {
            Val::Null => 0,
            Val::Int(i) => *i as u32,
            Val::Char(c) => *c,
            Val::String(_s) => todo!(),
        }
    }

    fn into_string(&self) -> String {
        match self {
            Val::Null => String::new(),
            Val::Int(i) => String::from(char::from_u32(*i as u32).unwrap_or_default()),
            Val::Char(c) => String::from(char::from_u32(*c).unwrap_or_default()),
            Val::String(s) => s.clone(),
        }
    }

    fn into_int(&self) -> i64 {
        match self {
            Val::Null => 0,
            Val::Int(i) => *i,
            Val::Char(_c) => todo!(),
            Val::String(_s) => todo!(),
        }
    }

    pub fn add(&mut self, val: Val) {
        //println!("self: {:?}, val: {:?}", self, val);
        match self {
            Val::Null => *self = val,
            Val::Int(i) => *i = *i + val.into_int(),
            Val::Char(c) => match val {
                Val::Int(_) | Val::Null => *c = *c + val.into_char(),
                Val::Char(_) | Val::String(_) => {
                    *self = Val::String(self.into_string());
                    self.add(val);
                }
            },
            Val::String(s) => s.push_str(val.into_string().as_str()),
        }
    }

    pub fn sub(&mut self, val: Val) {
        //println!("self: {:?}, val: {:?}", self, val);
        match self {
            Val::Null => todo!(),
            Val::Int(i) => *i = *i - val.into_int(),
            Val::Char(c) => *c = *c - val.into_char(),
            Val::String(_s) => todo!(),
        }
    }
}
