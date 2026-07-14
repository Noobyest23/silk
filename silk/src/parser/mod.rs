pub mod ast;

use crate::{lexer::token::{Token, TokenType}, parser::ast::{Program, expr::{ExprNode, SilkAssignment, SilkOperator}, stmt::StmtNode}};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement());
        }
        Program { statements }
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
    fn parse_statement(&mut self) -> StmtNode {
        match &self.peek().t {
            TokenType::Var => self.parse_var_declaration(),
            TokenType::Func => self.parse_func_declaration(),
            TokenType::Import => self.parse_import(),
            TokenType::Return => {
                self.advance();
                StmtNode::Return(self.parse_expression())
            }
            TokenType::If => self.parse_if(),
            TokenType::Global => {
                self.advance();
                StmtNode::Global(Box::new(self.parse_statement()))
            }
            TokenType::Struct => self.parse_struct(),
            _ => StmtNode::StandaloneExpression(self.parse_expression()),
        }
    }

    fn parse_var_declaration(&mut self) -> StmtNode {
        self.advance(); 

        let name = self.extract_id();

        let mut expr = ExprNode::NullLiteral;

        if self.check(TokenType::Equal) {
            self.advance();
            expr = self.parse_expression();
        }

        StmtNode::VarDecl(name, expr)
    }

    fn parse_func_declaration(&mut self) -> StmtNode {
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

        let mut statements: Vec<StmtNode> = Vec::new();
        self.expect(TokenType::OpenSquiggly); 
        while !self.is_at_end() && !self.check(TokenType::CloseSquiggly) {
            statements.push(self.parse_statement());
        }
        self.expect(TokenType::CloseSquiggly); 

        StmtNode::FuncDecl(name, args, statements)
    }

    fn parse_import(&mut self) -> StmtNode {
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

        StmtNode::Import(module_name, alias_name)
    }

    fn parse_if(&mut self) -> StmtNode {
        self.advance();
        let condition = self.parse_expression();
        let mut true_body = Vec::new();
        let mut false_body = Vec::new();
        self.expect(TokenType::OpenSquiggly);
        while !self.check(TokenType::CloseSquiggly) {
            true_body.push(self.parse_statement());
        }
        self.advance();
        if self.check(TokenType::Else) {
            self.advance();
            self.expect(TokenType::OpenSquiggly);
            while !self.check(TokenType::CloseSquiggly) {
                false_body.push(self.parse_statement());
            }
            self.advance();
        }

        StmtNode::If(condition, true_body, false_body)
    }

    fn parse_struct(&mut self) -> StmtNode {
        self.advance();
        let name = self.extract_id();
        self.expect(TokenType::OpenSquiggly);
        let mut struct_body = Vec::new();
        while !self.check(TokenType::CloseSquiggly) {
            let stmt = self.parse_statement();
            match stmt {
                StmtNode::VarDecl(_, _) => struct_body.push(stmt), 
                StmtNode::FuncDecl(_, _, _) => struct_body.push(stmt),
                _ => panic!("unexpected statement in struct body"),
            };
        }
        self.advance();
        return StmtNode::StructDecl(name, struct_body);
    }
}

impl Parser {
    fn parse_expression(&mut self) -> ExprNode {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ExprNode {
        
        let expr = self.parse_equality();

        
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

            
            let rhs = self.parse_assignment();

            return ExprNode::AssignmentOp(Box::new(expr), Box::new(rhs), op);
        }

        expr
    }

    fn parse_equality(&mut self) -> ExprNode {
        let mut expr = self.parse_boolean();

        while self.check(TokenType::DoubleEqual) {
            self.advance();
            let rhs = self.parse_boolean();
            expr = ExprNode::Op(Box::new(expr), Box::new(rhs), SilkOperator::Equality);
        }
        expr
    }

    fn parse_boolean(&mut self) -> ExprNode {
        let mut expr = self.parse_comparison();

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
            let rhs = self.parse_comparison();
            expr = ExprNode::Op(Box::new(expr), Box::new(rhs), op);
        }

        expr
    }

    fn parse_comparison(&mut self) -> ExprNode {
        let mut expr = self.parse_term();

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
            let rhs = self.parse_term();
            expr = ExprNode::Op(Box::new(expr), Box::new(rhs), op)
        }
        expr

    }

    fn parse_term(&mut self) -> ExprNode {
        let mut expr = self.parse_factor();
        
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
            let rhs = self.parse_factor();
            expr = ExprNode::Op(Box::new(expr), Box::new(rhs), op)
        }

        expr
    }

    fn parse_factor(&mut self) -> ExprNode {
        let mut expr = self.parse_unary();

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
            let rhs = self.parse_postfix();
            expr = ExprNode::Op(Box::new(expr), Box::new(rhs), op)
        }

        expr
    }

    fn parse_unary(&mut self) -> ExprNode {
        
        if self.check(TokenType::Minus) {
            self.advance(); 
            
            
            let rhs = self.parse_unary(); 
            return ExprNode::Unary(Box::new(rhs));
        }
        
        
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> ExprNode {
        let mut expr = self.parse_primary();

        loop {
            if self.check(TokenType::OpenParen) {
                self.advance(); 
                expr = self.finish_call(expr);
            } else if self.check(TokenType::OpenBracket) {
                self.advance(); 
                let index = self.parse_expression();
                self.expect(TokenType::CloseBracket);
                expr = ExprNode::IndexAccess(Box::new(expr), Box::new(index));
            } else if self.check(TokenType::Period) {
                self.advance(); 
                
                let name = self.extract_id();
                expr = ExprNode::DotAccess(Box::new(expr), Box::new(ExprNode::Var(name)));
            } else {
                break;
            }
        }
        expr
    }

    fn finish_call(&mut self, callee: ExprNode) -> ExprNode {
        let mut arguments = Vec::new();
        if !self.check(TokenType::CloseParen) {
            loop {
                arguments.push(self.parse_expression());
                if self.check(TokenType::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(TokenType::CloseParen);
        ExprNode::FuncCall(Box::new(callee), arguments)
    }

    fn parse_primary(&mut self) -> ExprNode {
            if self.check(TokenType::OpenParen) {
            self.advance(); 
            let expr = self.parse_expression();
            self.expect(TokenType::CloseParen); 
            return expr;
        }
        let tok = self.advance();
        match &tok.t {
            TokenType::Identifier(id) => ExprNode::Var(id.clone()),
            TokenType::IntLit(num) => ExprNode::IntLiteral(num.clone()),
            TokenType::FloatLit(num) => ExprNode::FloatLiteral(num.clone()),
            TokenType::StringLit(str) => ExprNode::StringLiteral(str.clone()),
            TokenType::Null => ExprNode::NullLiteral,
            TokenType::BoolLit(option) => ExprNode::BoolLiteral(option.clone()),
            TokenType::OpenBracket => {
                let mut array: Vec<ExprNode> = Vec::new();

                if !self.check(TokenType::CloseBracket) {
                    loop {
                        array.push(self.parse_expression());
                        if self.check(TokenType::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.advance();
                ExprNode::ArrayLiteral(array)
            },
            _ => {
                self.err("Unexpected Token");
                unreachable!();
            }
        }
    }

    
}
