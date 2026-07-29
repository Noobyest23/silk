use std::{collections::HashMap, fmt};
use crate::parser::ast::ProgramExpression;

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
            SilkOperator::Or 
            => "or",
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
    ArrayLiteral(Vec<ProgramExpression>),
    StructLiteral(HashMap<String, ProgramExpression>),
    NullLiteral,
    // expr is the function to be called, and the vector is the arguments
    FuncCall(ProgramExpression, Vec<ProgramExpression>),
    // rhs, lhs, operator
    Op(ProgramExpression, ProgramExpression, SilkOperator),
    AssignmentOp(ProgramExpression, ProgramExpression, SilkAssignment),
    Var(String),
    // owner, expression to evaluate after pushing scope
    DotAccess(ProgramExpression, ProgramExpression),
    // owner, index
    IndexAccess(ProgramExpression, ProgramExpression),
    // expression
    Unary(ProgramExpression),
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
                    write!(f, "{}", expr.node)?;
                }
                write!(f, "]")
            }
            ExprNode::StructLiteral(values) => {
                write!(f, "{{")?;
                for (key, value) in values {
                    write!(f, "{} : {}", key, value.node)?;
                }
                write!(f, "}}")
            }
            ExprNode::NullLiteral => write!(f, "null"),

            ExprNode::Op(lhs, rhs, op) => {
                // This handles the recursion automatically
                write!(f, "({} {} {})", lhs.node, op, rhs.node)
            }
            
            ExprNode::AssignmentOp(lhs, rhs, op) => {
                write!(f, "({} {} {})", lhs.node, op, rhs.node)
            }

            ExprNode::FuncCall(func, args) => {
                write!(f, "{}(", func.node)?; // Note the '?' for error propagation
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg.node)?;
                }
                write!(f, ")")
            }
            ExprNode::Var(id) => {
                write!(f, "{}", id)
            }
            ExprNode::DotAccess(owner, expr) => {
                write!(f, "{}.{}", owner.node, expr.node)
            }
            ExprNode::IndexAccess(owner, index) => {
                write!(f, "{}[{}]", owner.node, index.node)
            }
            ExprNode::Unary(expression) => {
                write!(f, "-{}", expression.node)
            }
        }
    }
}
