pub mod expr;
pub mod stmt;
use stmt::StmtNode;
use std::fmt;

use crate::parser::ast::expr::ExprNode;

pub struct Program {
    pub statements: Vec<ProgramStatement>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")?;
        for stmt in &self.statements {
            write!(f, "{}\n", stmt.node)?;
        }
        write!(f, "")
    }
}

#[derive(Clone, Debug)]
pub struct ProgramStatement {
    pub node: Box<StmtNode>,
    pub line: u32,
    pub column: u32,
}

impl ProgramStatement {
    pub fn new(node: StmtNode, ln: u32, col: u32) -> Self {
        Self {
            node: Box::new(node),
            line: ln,
            column: col,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProgramExpression {
    pub node: Box<ExprNode>,
    pub line: u32,
    pub column: u32,
}

impl ProgramExpression {
    pub fn new(node: ExprNode, ln: u32, col: u32) -> Self {
        Self {
            node: Box::new(node),
            line: ln,
            column: col,
        }
    }
}