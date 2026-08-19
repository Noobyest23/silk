pub mod ast;

use crate::{lexer::token::{Token, TokenType}, parser::ast::{Program, ProgramExpression, ProgramStatement, expr::{ExprNode::{self, Unary}, SilkAssignment, SilkOperator}, stmt::StmtNode::{self, VarDecl}}};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Option<Program> {
        let mut statements = Vec::new();
        statements.push(ProgramStatement { node: Box::new(StmtNode::Import("builtin".to_string(), "".to_string())), line: 0, column: 0 });
        while !self.is_at_end() {
            let res = self.parse_statement();
            match res {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    self.err(&e);
                    return None;
                }
            }
        }
        Some(Program { statements })
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().t == TokenType::Eof
    }

    fn check(&self, t: TokenType) -> bool {
        if self.is_at_end() { return false; }
        
        
        std::mem::discriminant(&self.peek().t) == std::mem::discriminant(&t)
    }

    fn expect(&mut self, t: TokenType) -> &Token {
        if self.is_at_end() {
            self.err("Unexpected End of File");
        }
        
        if std::mem::discriminant(&self.peek().t) != std::mem::discriminant(&t) {
            
            let msg = format!("Expected Token {}, but found {}", t, self.peek().t);
            self.err(&msg);
        }

        self.advance()
    }

    fn err(&mut self, what: &str) {
        println!("\x1b[31m[Parser Error]\x1b[0m {} at {} {} with a token of {} ", what, self.peek().line, self.peek().column, self.peek());
        self.current = self.tokens.len() - 1;
    }

    fn match_any(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn extract_id(&mut self) -> String {
        let tok = self.expect(TokenType::Identifier(String::new()));
        let id = if let TokenType::Identifier(name) = &tok.t {
            name.clone()
        } else {
            unreachable!("Expected identifier");
        };
        id
    }

}


impl Parser {
    fn parse_statement(&mut self) -> Result<ProgramStatement, String> {
        match &self.peek().t {
            TokenType::Var => self.parse_var_declaration(),
            TokenType::Func => self.parse_func_declaration(),
            TokenType::Import => self.parse_import(),
            TokenType::Return => {
                self.advance();
                let res = self.parse_expression();
                match res {
                    Ok(expr) => Ok(ProgramStatement::new(StmtNode::Return(expr.clone()), expr.line, expr.column)),
                    Err(msg) => Err(msg)
                }
            }
            TokenType::If => self.parse_if(),
            TokenType::Global => {
                self.advance();
                let res = self.parse_statement();
                match res {
                    Ok(expr) => Ok(ProgramStatement::new(StmtNode::Global(expr.clone()), expr.line, expr.column)),
                    Err(msg) => Err(msg)
                }
            }
            TokenType::Struct => self.parse_struct(),
            TokenType::For    => self.parse_for(),
            TokenType::While  => self.parse_while(),
            _ => {
                let res = self.parse_expression();
                match res {
                    Ok(expr) => Ok(ProgramStatement::new(StmtNode::StandaloneExpression(expr.clone()), expr.line, expr.column)),
                    Err(msg) => Err(msg)
                }
            }
        }
    }

    fn parse_var_declaration(&mut self) -> Result<ProgramStatement, String> {
        self.advance(); 

        let name = self.extract_id();

        let mut expr = ProgramExpression::new(ExprNode::NullLiteral, self.peek().line, self.peek().column);

        if self.check(TokenType::Equal) {
            self.advance();
            let res = self.parse_expression();
            match res {
                Ok(oexpr) => expr = oexpr,
                Err(e) => {return Err(e);}
            }
        }

        Ok(ProgramStatement::new(VarDecl(name, expr), self.peek().line, self.peek().column))
    }

    fn parse_func_declaration(&mut self) -> Result<ProgramStatement, String> {
        self.advance(); 
        let name = self.extract_id();

        self.expect(TokenType::OpenParen);
        let mut args: Vec<String> = Vec::new();
        if !self.check(TokenType::CloseParen) {
            loop {
                args.push(self.extract_id());
                if !self.match_any(&[TokenType::Comma]) { break; }
            }
        }
        self.expect(TokenType::CloseParen);

        let mut statements: Vec<ProgramStatement> = Vec::new();
        self.expect(TokenType::OpenSquiggly); 
        while !self.is_at_end() && !self.check(TokenType::CloseSquiggly) {
            let res = self.parse_statement();
            match res {
                Ok(stmt) => statements.push(stmt),
                Err(msg) => {return Err(msg);}
            }
        }
        self.expect(TokenType::CloseSquiggly); 

        Ok(ProgramStatement::new(StmtNode::FuncDecl(name, args, statements), self.peek().line, self.peek().column))
    }

    fn parse_import(&mut self) -> Result<ProgramStatement, String> {
        self.advance(); 
        
        
        let tok = self.expect(TokenType::StringLit(String::new()));
        let module_name = if let TokenType::StringLit(s) = &tok.t {
            s.clone()
        } else {
            unreachable!()
        };

        let mut alias_name = String::new();
        if self.check(TokenType::As) {
            self.advance();
            alias_name = self.extract_id();
        }

        Ok(ProgramStatement::new(StmtNode::Import(module_name, alias_name), self.peek().line, self.peek().column))
    }

    fn parse_if(&mut self) -> Result<ProgramStatement, String> {
        self.advance();
        let condition = self.parse_expression()?;

        let mut true_body = Vec::new();
        let mut false_body = Vec::new();
        self.expect(TokenType::OpenSquiggly);
        while !self.check(TokenType::CloseSquiggly) {
            let res = self.parse_statement();
            match res {
                Ok(stmt) => true_body.push(stmt),
                Err(msg) => {return Err(msg);}
            }
        }
        self.advance();
        if self.check(TokenType::Else) {
            self.advance();
            self.expect(TokenType::OpenSquiggly);
            while !self.check(TokenType::CloseSquiggly) {
                let res = self.parse_statement();
                match res {
                    Ok(stmt) => false_body.push(stmt),
                    Err(msg) => {return Err(msg);}
                }
            }
            self.advance();
        }

        Ok(ProgramStatement::new(StmtNode::If(condition, true_body, false_body), self.peek().line, self.peek().column))
    }

    fn parse_struct(&mut self) -> Result<ProgramStatement, String> {
        self.advance();
        let name = self.extract_id();
        self.expect(TokenType::OpenSquiggly);
        let mut struct_body = Vec::new();
        while !self.check(TokenType::CloseSquiggly) {
            let stmt = self.parse_statement()?;
            match stmt.node.as_ref() {
                StmtNode::VarDecl(_, _) => struct_body.push(stmt),
                StmtNode::FuncDecl(_, _, _) => struct_body.push(stmt),
                _ => {
                    return Err(format!("Unexpected statement in struct body '{}'", stmt.node));
                }
            };
        }
        self.advance();
        Ok(ProgramStatement::new(StmtNode::StructDecl(name, struct_body), self.peek().line, self.peek().column))
    }

    fn parse_for(&mut self) -> Result<ProgramStatement, String> {
        self.advance();
        let id = self.extract_id();
        self.advance();
        let container = self.parse_expression()?;

        let mut body = Vec::new();
        self.expect(TokenType::OpenSquiggly);
        while !self.check(TokenType::CloseSquiggly) {
            let stmt = self.parse_statement()?;
            body.push(stmt);
        }
        self.advance();

        Ok(ProgramStatement::new(StmtNode::For(id, container, body), self.peek().line, self.peek().column))
    }

    fn parse_while(&mut self) -> Result<ProgramStatement, String> {
        self.advance();
        let conditional = self.parse_expression()?;

        let mut body = Vec::new();
        self.expect(TokenType::OpenSquiggly);
        while !self.check(TokenType::CloseSquiggly) {
            let stmt = self.parse_statement()?;
            body.push(stmt);
        }
        self.advance();

        Ok(ProgramStatement::new(StmtNode::While(conditional, body), self.peek().line, self.peek().column))
    }

}

impl Parser {
    fn parse_expression(&mut self) -> Result<ProgramExpression, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<ProgramExpression, String> {
        let expr = self.parse_equality()?;

        if self.check(TokenType::Equal)
            || self.check(TokenType::PlusEq)
            || self.check(TokenType::MinusEq)
            || self.check(TokenType::MultiplyEq)
            || self.check(TokenType::DivideEq)
            || self.check(TokenType::ModEq) {
            let tok = self.advance();

            let op = match &tok.t {
                TokenType::Equal => SilkAssignment::Assignment,
                TokenType::PlusEq => SilkAssignment::CompoundPlus,
                TokenType::MinusEq => SilkAssignment::CompoundMinus,
                TokenType::MultiplyEq => SilkAssignment::CompoundMultiply,
                TokenType::DivideEq => SilkAssignment::CompoundDivide,
                TokenType::ModEq => SilkAssignment::CompoundMod,
                _ => unreachable!(),
            };

            let rhs = self.parse_assignment()?;

            return Ok(ProgramExpression::new(ExprNode::AssignmentOp(expr, rhs, op), self.peek().line, self.peek().column));
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<ProgramExpression, String> {
        let mut expr = self.parse_boolean()?;

        while self.check(TokenType::DoubleEqual) || self.check(TokenType::NotEqual) {
            let operator = match self.peek().t {
                TokenType::DoubleEqual => SilkOperator::Equality,
                TokenType::NotEqual    => SilkOperator::NotEqual,
                _ => unreachable!()
            };
            self.advance();
            let rhs = self.parse_boolean()?;
            expr = ProgramExpression::new(ExprNode::Op(expr, rhs, operator), self.peek().line, self.peek().column);
        }
        Ok(expr)
    }

    fn parse_boolean(&mut self) -> Result<ProgramExpression, String> {
        let mut expr = self.parse_comparison()?;

        while self.check(TokenType::And) || self.check(TokenType::Or) {
            let tok = self.advance();
            let op = match &tok.t {
                TokenType::And => SilkOperator::And,
                TokenType::Or => SilkOperator::Or,
                _ => {
                    self.err("Expected a boolean operator");
                    unreachable!()
                }
            };
            let rhs = self.parse_comparison()?;
            expr = ProgramExpression::new(ExprNode::Op(expr, rhs, op), self.peek().line, self.peek().column);
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<ProgramExpression, String> {
        let mut expr = self.parse_term()?;

        while self.check(TokenType::GreaterThan) || self.check(TokenType::GreaterThanEq) || self.check(TokenType::LesserThan) || self.check(TokenType::LesserThanEq) {
            let tok = self.advance();
            let op: SilkOperator = match &tok.t {
                TokenType::GreaterThan => SilkOperator::GreaterThan,
                TokenType::LesserThan => SilkOperator::LesserThan,
                TokenType::GreaterThanEq => SilkOperator::GreaterThanEq,
                TokenType::LesserThanEq => SilkOperator::LesserThanEq,
                _ => {
                    self.err("Expected an operator");
                    unreachable!();
                }
            };
            let rhs = self.parse_term()?;
            expr = ProgramExpression::new(ExprNode::Op(expr, rhs, op), self.peek().line, self.peek().column)
        }
        Ok(expr)

    }

    fn parse_term(&mut self) -> Result<ProgramExpression, String> {
        let mut expr = self.parse_factor()?;

        while self.check(TokenType::Plus) || self.check(TokenType::Minus) {
            let tok = self.advance();
            let op: SilkOperator = match &tok.t {
                TokenType::Plus => SilkOperator::Plus,
                TokenType::Minus => SilkOperator::Minus,
                _ => {
                    self.err("Expected an operator");
                    unreachable!();
                }
            };
            let rhs = self.parse_factor()?;
            expr = ProgramExpression::new(ExprNode::Op(expr, rhs, op), self.peek().line, self.peek().column)
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<ProgramExpression, String> {
        let mut expr = self.parse_unary()?;

        while self.check(TokenType::Asterisk) || self.check(TokenType::FrontSlash) || self.check(TokenType::Percent) {
            let tok = self.advance();
            let op: SilkOperator = match &tok.t {
                TokenType::Asterisk => SilkOperator::Multiply,
                TokenType::FrontSlash => SilkOperator::Divide,
                TokenType::Percent => SilkOperator::Mod,
                _ => {
                    self.err("Expected an operator");
                    unreachable!();
                }
            };
            let rhs = self.parse_postfix()?;
            expr = ProgramExpression::new(ExprNode::Op(expr, rhs, op), self.peek().line, self.peek().column)
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<ProgramExpression, String> {
        if self.check(TokenType::Minus) {
            self.advance();
            let result = self.parse_unary()?;
            Ok(ProgramExpression::new(Unary(result), self.peek().line, self.peek().column))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<ProgramExpression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(TokenType::OpenParen) {
                self.advance();
                expr = self.finish_call(expr)?;
            } else if self.check(TokenType::OpenBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.expect(TokenType::CloseBracket);
                expr = ProgramExpression::new(ExprNode::IndexAccess(expr, index), self.peek().line, self.peek().column);
            } else if self.check(TokenType::Period) {
                self.advance();

                let name = self.extract_id();
                let rhs = ProgramExpression::new(ExprNode::Var(name), self.peek().line, self.peek().column);
                expr = ProgramExpression::new(ExprNode::DotAccess(expr, rhs), self.peek().line, self.peek().column);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: ProgramExpression) -> Result<ProgramExpression, String> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::CloseParen) {
            loop {
                let res = self.parse_expression();
                match res {
                    Ok(expr) => arguments.push(expr),
                    Err(what) => {return Err(what);}
                }
                if self.check(TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenType::CloseParen);
        Ok(ProgramExpression::new(ExprNode::FuncCall(callee, arguments), self.peek().line, self.peek().column))
    }

    fn parse_primary(&mut self) -> Result<ProgramExpression, String> {
            if self.check(TokenType::OpenParen) {
            self.advance(); 
            let expr = self.parse_expression();
            self.expect(TokenType::CloseParen); 
            return expr;
        }
        let tok = self.advance();
        match &tok.t {
            TokenType::Identifier(id) => Ok(ProgramExpression::new(ExprNode::Var(id.clone()), self.peek().line, self.peek().column)),
            TokenType::IntLit(num) => Ok(ProgramExpression::new(ExprNode::IntLiteral(num.clone()), self.peek().line, self.peek().column)),
            TokenType::FloatLit(num) => Ok(ProgramExpression::new(ExprNode::FloatLiteral(num.clone()), self.peek().line, self.peek().column)),
            TokenType::StringLit(str) => Ok(ProgramExpression::new(ExprNode::StringLiteral(str.clone()), self.peek().line, self.peek().column)),
            TokenType::Null => Ok(ProgramExpression::new(ExprNode::NullLiteral, self.peek().line, self.peek().column)),
            TokenType::BoolLit(option) => Ok(ProgramExpression::new(ExprNode::BoolLiteral(option.clone()), self.peek().line, self.peek().column)),
            TokenType::OpenBracket => {
                let mut array: Vec<ProgramExpression> = Vec::new();

                if !self.check(TokenType::CloseBracket) {
                    loop {
                        let res = self.parse_expression();
                        match res {
                            Ok(expr) => array.push(expr),
                            Err(what) => {return Err(what);}
                        }
                        if self.check(TokenType::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.advance();
                Ok(ProgramExpression::new(ExprNode::ArrayLiteral(array), self.peek().line, self.peek().column))
            },
            _ => {
                Err(format!("Unexpected token in expression '{}'", tok))
            }
        }
    }

    
}
