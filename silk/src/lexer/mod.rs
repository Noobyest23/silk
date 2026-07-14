use crate::lexer::token::{Token, TokenType};
pub mod token;
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: u32,
    column: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn make_token(&self, t: TokenType) -> Token {
        Token {
            t,
            line: self.line,
            column: self.column,
        }
    }

    fn err(&mut self, what: &str) {
        println!("\x1b[31m[Lexer Error]\x1b[0m {} at {}, {}", what, self.line, self.column);
        while self.chars.peek().is_some() {
            self.advance();
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(&c) = self.chars.peek() {
            match c {
                
                ' ' | '\r' | '\t' | '\n' => {
                    self.advance();
                }
                
                
                '#' => {
                    while let Some(&ch) = self.chars.peek() {
                        if ch == '\n' { break; }
                        self.advance();
                    }
                    continue;
                }
                '!' => {
                    let tok = self.make_token(TokenType::Not);
                    self.advance();
                    tokens.push(tok);
                }
                '(' => {
                    let tok = self.make_token(TokenType::OpenParen);
                    self.advance();
                    tokens.push(tok);
                }
                ')' => { 
                    let tok = self.make_token(TokenType::CloseParen);
                    self.advance();
                    tokens.push(tok);
                }
                '{' => {
                    let tok = self.make_token(TokenType::OpenSquiggly);
                    self.advance();
                    tokens.push(tok);
                }
                '}' => {
                    let tok = self.make_token(TokenType::CloseSquiggly);
                    self.advance();
                    tokens.push(tok);
                }
                '[' => {
                    let tok = self.make_token(TokenType::OpenBracket);
                    self.advance();
                    tokens.push(tok);
                }
                ']' => {
                    let tok = self.make_token(TokenType::CloseBracket);
                    self.advance();
                    tokens.push(tok);
                }
                '+' => { 
                    let start_col = self.column;
                    self.advance();
                    if self.match_next('=') {
                        tokens.push(Token { t: TokenType::PlusEq, line: self.line, column: start_col });
                    } else {
                        tokens.push(Token { t: TokenType::Plus, line: self.line, column: start_col });
                    }
                }
                '-' => {
                    let start_col = self.column;
                    self.advance();
                    if self.match_next('=') {
                        tokens.push(Token { t: TokenType::MinusEq, line: self.line, column: start_col });
                    } else {
                        tokens.push(Token { t: TokenType::Minus, line: self.line, column: start_col });
                    }
                }
                '*' => {
                    let start_col = self.column;
                    self.advance();
                    if self.match_next('=') {
                        tokens.push(Token { t: TokenType::MultiplyEq, line: self.line, column: start_col });
                    } else {
                        tokens.push(Token { t: TokenType::Asterisk, line: self.line, column: start_col });
                    }
                }
                '/' => {
                    let start_col = self.column;
                    self.advance();
                    if self.match_next('=') {
                        tokens.push(Token { t: TokenType::DivideEq, line: self.line, column: start_col });
                    } else {
                        tokens.push(Token { t: TokenType::FrontSlash, line: self.line, column: start_col });
                    }
                }
                '%' => {
                    let start_col = self.column;
                    self.advance();
                    if self.match_next('=') {
                        tokens.push(Token { t: TokenType::DivideEq, line: self.line, column: start_col });
                    } else {
                        tokens.push(Token { t: TokenType::Percent, line: self.line, column: start_col });
                    }
                }
                '&' => {
                    self.advance();
                    let token_type = self.expect_next('&', TokenType::And);
                    tokens.push(self.make_token(token_type));
                    
                }
                '|' => {
                    self.advance();
                    let token_type = self.expect_next('|', TokenType::Or);
                    tokens.push(self.make_token(token_type));
                }
                '=' => {
                    let start_col = self.column;
                    self.advance();
                    if self.match_next('=') {
                        tokens.push(Token { t: TokenType::DoubleEqual, line: self.line, column: start_col });
                    } else {
                        tokens.push(Token { t: TokenType::Equal, line: self.line, column: start_col });
                    }
                }
                ',' => {
                    self.advance();
                    tokens.push(self.make_token(TokenType::Comma));
                }
                '.' => {
                    self.advance();
                    tokens.push(self.make_token(TokenType::Period));
                }
                '>' => {
                    self.advance();
                    if self.match_next('=') {
                        tokens.push(self.make_token(TokenType::GreaterThanEq));
                    }
                    else {
                        tokens.push(self.make_token(TokenType::GreaterThan));
                    }
                }
                '<' => {
                    self.advance();
                    if self.match_next('=') {
                        tokens.push(self.make_token(TokenType::LesserThanEq));
                    }
                    else {
                        tokens.push(self.make_token(TokenType::LesserThan));
                    }
                }
                
                '"' => tokens.push(self.read_string()),

                // Identifiers or Keywords
                c if c.is_alphabetic() || c == '_' => tokens.push(self.read_identifier()),

                // Numbers
                c if c.is_numeric() => tokens.push(self.read_number()),

                _ => { 
                    eprintln!("Unexpected character: {}", c);
                    self.advance();
                }
            }
        }
        
        tokens.push(self.make_token(TokenType::Eof));
        tokens
    }
}

impl<'a> Lexer<'a> {
    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        match ident.as_str() {
            "func"   => self.make_token(TokenType::Func),
            "var"    => self.make_token(TokenType::Var),
            "import" => self.make_token(TokenType::Import),
            "true"   => self.make_token(TokenType::BoolLit(true)),
            "false"  => self.make_token(TokenType::BoolLit(false)),
            "and"    => self.make_token(TokenType::And),
            "or"     => self.make_token(TokenType::Or),
            "not"    => self.make_token(TokenType::Not),
            "const"  => self.make_token(TokenType::Const),
            "for"    => self.make_token(TokenType::For),
            "while"  => self.make_token(TokenType::While),
            "as"     => self.make_token(TokenType::As),
            "if"     => self.make_token(TokenType::If),
            "else"   => self.make_token(TokenType::Else),
            "null"   => self.make_token(TokenType::Null),
            "return" => self.make_token(TokenType::Return),
            "global" => self.make_token(TokenType::Global),
            "struct" => self.make_token(TokenType::Struct),
            _        => self.make_token(TokenType::Identifier(ident)),
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() {
                num_str.push(self.advance().unwrap());
            } else if c == '.' && !is_float {
                is_float = true;
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        if is_float {
            self.make_token(TokenType::FloatLit(num_str.parse().unwrap_or(0.0)))
        } else {
            self.make_token(TokenType::IntLit(num_str.parse().unwrap_or(0)))
        }
    }

    fn read_string(&mut self) -> Token {
        let start_line = self.line;
        let start_column = self.column;

        self.advance(); // Consume the opening '"'
        let mut string = String::new();

        while let Some(&c) = self.chars.peek() {
            match c {
                '\\' => { 
                    self.advance(); 
                    if let Some(escaped) = self.advance() {
                        match escaped {
                            'n' => string.push('\n'),
                            't' => string.push('\t'),
                            'r' => string.push('\r'),
                            '\\' => string.push('\\'),
                            '"' => string.push('"'),
                            _ => {
                                
                                eprintln!("[Lexer Error] Invalid escape sequence '\\{}' at {}:{}", escaped, self.line, self.column);
                                string.push(escaped); 
                            }
                        }
                    }
                }
                '"' => {
                    self.advance(); // Consume closing '"'
                    return Token {
                        t: TokenType::StringLit(string),
                        line: start_line,
                        column: start_column,
                    };
                }
                _ => {
                    string.push(self.advance().unwrap());
                }
            }
        }
        
        self.err("Unterminated string literal");
        Token { t: TokenType::Eof, line: start_line, column: start_column }
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.chars.peek() == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_next(&mut self, expected: char, t: TokenType) -> TokenType {
        if self.match_next(expected) {
            t
        } else {
            
            let next_ch = self.chars.peek().copied().unwrap_or(' ');
            let msg = format!("Expected '{}' after '{}' at {}:{}", expected, next_ch, self.line, self.column);
            self.err(&msg);
            TokenType::Eof
        }
    }
}

