#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn is_i64(&self) -> bool {
        match self {
            Value::Number(n) => n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64,
            _ => false,
        }
    }

    pub fn is_u64(&self) -> bool {
        match self {
            Value::Number(n) => n.fract() == 0.0 && *n >= 0.0 && *n <= u64::MAX as f64,
            _ => false,
        }
    }

    pub fn is_f64(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(n) if n.fract() == 0.0 => Some(*n as i64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(n) if n.fract() == 0.0 && *n >= 0.0 => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", v)?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in obj {
                    if !first {
                        write!(f, ", ")?;
                    }
                    first = false;
                    write!(f, "\"{}\": {:?}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(n as f64)
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::Number(n as f64)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<Vec<Value>> for Value {
    fn from(arr: Vec<Value>) -> Self {
        Value::Array(arr)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(obj: HashMap<String, Value>) -> Self {
        Value::Object(obj)
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}
