use std::fmt::Display;

use crate::common::datatypes::Variable;

use super::{keywords::Keyword, symbols::Symbol};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.kind)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    LiteralToken(Variable),
    /// number of whitespace
    WhitespaceToken(usize),
    NewLineToken,
    KeywordToken(Keyword),
    SymbolToken(Symbol),
    FactoryToken,
    IdentifierToken(String),
    EndOfFileToken,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::LiteralToken(a) => write!(f, "{}", a),
            TokenKind::WhitespaceToken(a) => write!(f, "{}", a),
            TokenKind::NewLineToken => write!(f, "NewLineToken"),
            TokenKind::KeywordToken(a) => write!(f, "{}", a),
            TokenKind::SymbolToken(a) => write!(f, "{}", a),
            TokenKind::FactoryToken => write!(f, "FactoryToken"),
            TokenKind::IdentifierToken(a) => write!(f, "{}", a),
            TokenKind::EndOfFileToken => write!(f, "EndOfFileToken"),
        }
    }
}
