use super::expr::ExprNode;
use std::fmt;

#[derive(Clone, Debug)]
pub enum StmtNode {
    VarDecl(String, ExprNode),
    FuncDecl(String, Vec<String>, Vec<StmtNode>),
    StructDecl(String, Vec<StmtNode>), // struct foo {var bar = 10 func read_bar() {return bar}}
    Import(String, String), // optional import as
    StandaloneExpression(ExprNode),
    Return(ExprNode),
    If(ExprNode, Vec<StmtNode>, Vec<StmtNode>),
    Global(Box<StmtNode>), // global declaration. global var blah = 2031049102
}

impl fmt::Display for StmtNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StmtNode::VarDecl(name, expr) => write!(f, "var {} = {}", name, expr),
            StmtNode::FuncDecl(name, args, body) => {
                write!(f, "func {}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {write!(f, ", ")?;}
                    write!(f, "{}", arg)?;
                }
                write!(f, ") {{")?;
                for (_i, stmt) in body.iter().enumerate() {
                    write!(f, "{}\n", stmt)?;
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
                    write!(f, "{}\n", stmt)?;
                }
                write!(f, "}}")
            }
            StmtNode::StandaloneExpression(expr) => write!(f, "(standalone) {}", expr),
            StmtNode::Return(value) => write!(f, "return {}", value),
            StmtNode::If(condition, truthy, falsy) => {
                write!(f, "if {} '{{'\n", condition);
                for truthy_stmt in truthy {
                    write!(f, "{}\n", truthy_stmt);
                }
                write!(f, "else '{{'\n");
                for falsy_stmt in falsy {
                    write!(f, "{}\n", falsy_stmt);
                }
                write!(f, "}}")
            }
            StmtNode::Global(stmt) => {
                write!(f, "global {}", stmt)
            }
        }
    }
}