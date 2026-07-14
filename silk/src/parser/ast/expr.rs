use std::{collections::HashMap, fmt};

#[derive(Clone, Debug)]
pub enum SilkOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Mod,
    Equality,
    GreaterThan,
    LesserThan,
    GreaterThanEq,
    LesserThanEq,
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum SilkAssignment {
    Assignment,
    CompoundPlus,
    CompoundMinus,
    CompoundMultiply,
    CompoundDivide,
    CompoundMod,
}


impl fmt::Display for SilkOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SilkOperator::Plus => "+",
            SilkOperator::Minus => "-",
            SilkOperator::Multiply => "*",
            SilkOperator::Divide => "/",
            SilkOperator::Mod => "%",
            SilkOperator::Equality => "==",
            SilkOperator::GreaterThan => ">",
            SilkOperator::LesserThan => "<",
            SilkOperator::GreaterThanEq => ">=",
            SilkOperator::LesserThanEq => "<=",
            SilkOperator::And => "and",
            SilkOperator::Or => "or",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for SilkAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SilkAssignment::Assignment => "=",
            SilkAssignment::CompoundDivide => "/=",
            SilkAssignment::CompoundMinus => "-=",
            SilkAssignment::CompoundMod => "%=",
            SilkAssignment::CompoundMultiply => "*=",
            SilkAssignment::CompoundPlus => "+=",
        };
        write!(f, "{}", s)
    }
}


#[derive(Clone, Debug)]
pub enum ExprNode {
    IntLiteral(i32),
    FloatLiteral(f32),
    BoolLiteral(bool),
    StringLiteral(String),
    ArrayLiteral(Vec<ExprNode>),
    StructLiteral(HashMap<String, ExprNode>),
    NullLiteral,
    // expr is the function to be called, and the vector is the arguments
    FuncCall(Box<ExprNode>, Vec<ExprNode>),
    // rhs, lhs, operator
    Op(Box<ExprNode>, Box<ExprNode>, SilkOperator),
    AssignmentOp(Box<ExprNode>, Box<ExprNode>, SilkAssignment),
    Var(String),
    // owner, expression to evaluate after pushing scope
    DotAccess(Box<ExprNode>, Box<ExprNode>),
    // owner, index
    IndexAccess(Box<ExprNode>, Box<ExprNode>),
    // expression
    Unary(Box<ExprNode>),
}

impl fmt::Display for ExprNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprNode::IntLiteral(i) => write!(f, "{}", i),
            ExprNode::FloatLiteral(fl) => write!(f, "{}", fl),
            ExprNode::BoolLiteral(b) => write!(f, "{}", b),
            ExprNode::StringLiteral(s) => write!(f, "\"{}\"", s),
            ExprNode::ArrayLiteral(arr) => {
                write!(f, "[")?;
                for (i, expr) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?;}
                    write!(f, "{}", expr)?;
                }
                write!(f, "]")
            }
            ExprNode::StructLiteral(values) => {
                write!(f, "{{")?;
                for (key, value) in values {
                    write!(f, "{} : {}", key, value)?;
                }
                write!(f, "}}")
            }
            ExprNode::NullLiteral => write!(f, "null"),

            ExprNode::Op(lhs, rhs, op) => {
                // This handles the recursion automatically
                write!(f, "({} {} {})", lhs, op, rhs)
            }
            
            ExprNode::AssignmentOp(lhs, rhs, op) => {
                write!(f, "({} {} {})", lhs, op, rhs)
            }

            ExprNode::FuncCall(func, args) => {
                write!(f, "{}(", func)?; // Note the '?' for error propagation
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ExprNode::Var(id) => {
                write!(f, "{}", id)
            }
            ExprNode::DotAccess(owner, expr) => {
                write!(f, "{}.{}", owner, expr)
            }
            ExprNode::IndexAccess(owner, index) => {
                write!(f, "{}[{}]", owner, index)
            }
            ExprNode::Unary(expression) => {
                write!(f, "-{}", expression)
            }
        }
    }
}