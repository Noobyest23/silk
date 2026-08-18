use crate::parser::ast::{ProgramExpression, ProgramStatement};

use std::fmt;

#[derive(Clone, Debug)]
pub enum StmtNode {
    VarDecl(String, ProgramExpression),
    FuncDecl(String, Vec<String>, Vec<ProgramStatement>),
    StructDecl(String, Vec<ProgramStatement>), // struct foo {var bar = 10 func read_bar() {return bar}}
    Import(String, String), // optional import as
    StandaloneExpression(ProgramExpression),
    Return(ProgramExpression),
    If(ProgramExpression, Vec<ProgramStatement>, Vec<ProgramStatement>),
    Global(ProgramStatement), // global declaration. global var blah = 2031049102
}

impl fmt::Display for StmtNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StmtNode::VarDecl(name, expr) => write!(f, "var {} = {}", name, expr.node),
            StmtNode::FuncDecl(name, args, body) => {
                write!(f, "func {}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {write!(f, ", ")?;}
                    write!(f, "{}", arg)?;
                }
                write!(f, ") {{")?;
                for (_i, stmt) in body.iter().enumerate() {
                    write!(f, "{}\n", stmt.node)?;
                }
                write!(f, "}}")
            }
            StmtNode::Import(module,alias ) => {
                if alias != "" {
                    write!(f, "import {} as {}", module, alias)
                }
                else {
                    write!(f, "import {}", module)
                }
            }
            StmtNode::StructDecl(name, body) => {
                write!(f, "func {}", name)?;
                write!(f, " {{")?;
                for (_i, stmt) in body.iter().enumerate() {
                    write!(f, "{}\n", stmt.node)?;
                }
                write!(f, "}}")
            }
            StmtNode::StandaloneExpression(expr) => write!(f, "(standalone) {}", expr.node),
            StmtNode::Return(value) => write!(f, "return {}", value.node),
            StmtNode::If(condition, truthy, falsy) => {
                write!(f, "if {} '{{'\n", condition.node);
                for truthy_stmt in truthy {
                    write!(f, "{}\n", truthy_stmt.node);
                }
                write!(f, "else '{{'\n");
                for falsy_stmt in falsy {
                    write!(f, "{}\n", falsy_stmt.node);
                }
                write!(f, "}}")
            }
            StmtNode::Global(stmt) => {
                write!(f, "global {}", stmt.node)
            }
        }
    }
}