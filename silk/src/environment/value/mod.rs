use std::collections::HashMap;
use std::{fmt};
use std::cmp::Ordering;
use crate::environment::vm::{VirtualMachine};
use crate::parser::ast::stmt::StmtNode;

pub type NativeFn = fn (vm: &mut VirtualMachine, &Vec<SilkValue>) -> SilkValue;

#[derive(Clone, Debug)]
pub enum SilkValue {
    Null,
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Object(HashMap<String, SilkValue>),
    List(Vec<SilkValue>),
    Function(Vec<String>, Vec<StmtNode>),
    NativeFn(NativeFn),
    Pointer(usize),
    ObjectDefinition(Vec<StmtNode>),
}

impl fmt::Display for SilkValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SilkValue::Null => write!(f, "null"),
            SilkValue::Int(num) => write!(f, "{}", num),
            SilkValue::Float(num) => write!(f, "{}", num),
            SilkValue::Bool(b) => write!(f, "{}", if *b {"true"} else {"false"}),
            SilkValue::String(str) => write!(f, "{}", str),
            SilkValue::Object(obj) => {
                write!(f, "{{");
                for (key, value) in obj {
                    write!(f, "{} : {}", key, value);
                }
                write!(f, "}}")
            }
            SilkValue::List(list) => {
                write!(f, "[")?;
                for (i, value) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            },
            SilkValue::Function(args, body) => {
                write!(f, "({}) {{", args.join(", "))?;
                for stmt in body {
                    write!(f, "\n    {}", stmt)?;
                }
                write!(f, "\n}}")
            },
            SilkValue::NativeFn(_) => {
                write!(f, "(Cannot Print Native functions)")
            },
            SilkValue::Pointer(ptr) => {
                write!(f, "ptr({})", ptr)
            },
            SilkValue::ObjectDefinition(def) => {
                write!(f, "Struct Definition {{")?;
                for stmt in def {
                    write!(f, "\n    {}", stmt)?;
                }
                write!(f, "\n}}")
            }
        }
    }
}

impl SilkValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            SilkValue::Null => false,
            SilkValue::Bool(b) => *b,
            SilkValue::Int(i) => *i != 0,
            SilkValue::Float(f) => *f != 0.0,
            SilkValue::String(s) => !s.is_empty(),
            SilkValue::List(l) => !l.is_empty(),
            SilkValue::Function(_, _) => true,
            SilkValue::NativeFn(_) => true,
            SilkValue::Object(_) => true,
            SilkValue::Pointer(ptr) => *ptr == 0 as usize,
            SilkValue::ObjectDefinition(_) => false,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            SilkValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            SilkValue::Float(f) => Some(*f),
            SilkValue::Int(i) => Some(*i as f32),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            SilkValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn equals(&self, other: &SilkValue) -> bool {
        match (self, other) {
            (SilkValue::Null, SilkValue::Null) => true,
            (SilkValue::Int(a), SilkValue::Int(b)) => a == b,
            (SilkValue::Float(a), SilkValue::Float(b)) => a == b,
            (SilkValue::Int(a), SilkValue::Float(b)) => (*a as f32) == *b,
            (SilkValue::Float(a), SilkValue::Int(b)) => *a == (*b as f32),
            (SilkValue::Bool(a), SilkValue::Bool(b)) => a == b,
            (SilkValue::String(a), SilkValue::String(b)) => a == b,
            (SilkValue::List(la), SilkValue::List(lb)) => {
                if la.len() != lb.len() { return false; }
                for (x, y) in la.iter().zip(lb.iter()) {
                    if !x.equals(y) { return false; }
                }
                true
            }
            _ => false,
        }
    }

    pub fn compare(&self, other: &SilkValue) -> Option<Ordering> {
        match (self, other) {
            (SilkValue::Int(a), SilkValue::Int(b)) => Some(a.cmp(b)),
            (SilkValue::Float(a), SilkValue::Float(b)) => {
                if a < b { Some(Ordering::Less) } else if a > b { Some(Ordering::Greater) } else { Some(Ordering::Equal) }
            }
            (SilkValue::Int(a), SilkValue::Float(b)) => {
                let af = *a as f32;
                if af < *b { Some(Ordering::Less) } else if af > *b { Some(Ordering::Greater) } else { Some(Ordering::Equal) }
            }
            (SilkValue::Float(a), SilkValue::Int(b)) => {
                let bf = *b as f32;
                if *a < bf { Some(Ordering::Less) } else if *a > bf { Some(Ordering::Greater) } else { Some(Ordering::Equal) }
            }
            (SilkValue::String(a), SilkValue::String(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }

    pub fn is_type(&self, other: &SilkValue) -> bool {
        match (self, other) {
            (SilkValue::Null, SilkValue::Null) => true,
            (SilkValue::Int(_), SilkValue::Int(_)) => true,
            (SilkValue::Float(_), SilkValue::Float(_)) => true,
            
            (SilkValue::Int(_), SilkValue::Float(_)) => true,
            (SilkValue::Float(_), SilkValue::Int(_)) => true,
            (SilkValue::Bool(_), SilkValue::Bool(_)) => true,
            (SilkValue::String(_), SilkValue::String(_)) => true,
            (SilkValue::List(_), SilkValue::List(_)) => true,
            (SilkValue::Function(_, _), SilkValue::Function(_, _)) => true,
            _ => false,
        }
    }
}