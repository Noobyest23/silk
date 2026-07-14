
#[derive(PartialEq, Clone)]
pub enum TokenType {
    Identifier(String),
    IntLit(i32),
    FloatLit(f32),
    BoolLit(bool),
    StringLit(String),

    Plus,
    Minus,
    Asterisk,
    FrontSlash,
    Percent,
    DoubleEqual,
    GreaterThan,
    GreaterThanEq,
    LesserThan,
    LesserThanEq,
    Not,

    Equal,
    PlusEq,
    MinusEq,
    MultiplyEq,
    DivideEq,
    ModEq,

    OpenParen,
    CloseParen,
    OpenSquiggly,
    CloseSquiggly,
    OpenBracket,
    CloseBracket,
    Comma,
    Period,

    Import,
    Var,
    Func,
    Const,
    As,
    For,
    While,
    And,
    Or,
    If,
    Else,
    Null,
    Return,
    Global,
    Struct,

    Eof,
}

use std::fmt;

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            
            TokenType::Identifier(s) => write!(f, "{}", s),
            TokenType::IntLit(i)     => write!(f, "{}", i),
            TokenType::FloatLit(fl)  => write!(f, "{}", fl),
            TokenType::BoolLit(b)    => write!(f, "{}", b),
            TokenType::StringLit(s)  => write!(f, "\"{}\"", s),

            
            TokenType::Plus       => write!(f, "+"),
            TokenType::Minus      => write!(f, "-"),
            TokenType::Asterisk   => write!(f, "*"),
            TokenType::FrontSlash => write!(f, "/"),
            TokenType::Percent    => write!(f, "%"),
            TokenType::DoubleEqual    => write!(f, "=="),
            TokenType::GreaterThan => write!(f, ">"),
            TokenType::GreaterThanEq => write!(f, ">="),
            TokenType::LesserThan => write!(f, "<"),
            TokenType::LesserThanEq => write!(f, "<="),
            TokenType::Not => write!(f, "!"),

            
            TokenType::Equal    => write!(f, "="),
            TokenType::PlusEq   => write!(f, "+="),
            TokenType::MinusEq  => write!(f, "-="),
            TokenType::MultiplyEq => write!(f, "*="),
            TokenType::DivideEq => write!(f, "/="),
            TokenType::ModEq => write!(f, "%="),

            
            TokenType::OpenParen     => write!(f, "("),
            TokenType::CloseParen    => write!(f, ")"),
            TokenType::OpenSquiggly  => write!(f, "{{"),
            TokenType::CloseSquiggly => write!(f, "}}"),
            TokenType::OpenBracket   => write!(f, "["),
            TokenType::CloseBracket  => write!(f, "]"),
            TokenType::Comma         => write!(f, ","),
            TokenType::Period        => write!(f, "."),

            
            TokenType::Import => write!(f, "import"),
            TokenType::Var    => write!(f, "var"),
            TokenType::Func   => write!(f, "func"),
            TokenType::Const  => write!(f, "const"),
            TokenType::As     => write!(f, "as"),
            TokenType::For    => write!(f, "for"),
            TokenType::While  => write!(f, "while"),
            TokenType::And    => write!(f, "and"),
            TokenType::Or     => write!(f, "or"),
            TokenType::If     => write!(f, "if"),
            TokenType::Else   => write!(f, "else"),
            TokenType::Null   => write!(f, "null"),
            TokenType::Return => write!(f, "return"),
            TokenType::Global => write!(f, "global"),
            TokenType::Struct => write!(f, "struct"),

            TokenType::Eof => write!(f, "End of file"),
        }
    }
}

pub struct Token {
    pub t: TokenType,
    pub line: u32,
    pub column: u32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        
        write!(f, "{}", self.t)
    }
}
