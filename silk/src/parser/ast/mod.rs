pub mod expr;
pub mod stmt;
use stmt::StmtNode;
use std::fmt;

pub struct Program {
    pub statements: Vec<StmtNode>,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")?;
        for stmt in &self.statements {
            write!(f, "{}\n", stmt)?;
        }
        write!(f, "")
    }
}