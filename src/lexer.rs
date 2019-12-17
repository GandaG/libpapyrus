use std::str::{Bytes, FromStr};

use strum_macros::EnumString;

use crate::{Game, ParserSession};

const EOF_CHAR: u8 = b'\0';

fn is_whitespace(byte: u8) -> bool {
    match byte {
        b' ' | b'\t' => true,
        _ => false,
    }
}

fn is_newline_start(byte: u8) -> bool {
    match byte {
        b'\r' | b'\n' => true,
        _ => false,
    }
}

fn is_id_start(byte: u8) -> bool {
    (b'a' <= byte && byte <= b'z') || (b'A' <= byte && byte <= b'Z') || byte == b'_'
}

fn is_id_continue(byte: u8, game: &Game) -> bool {
    is_id_start(byte) || (b'0' <= byte && byte <= b'9') || (*game == Game::FO4 && byte == b':')
}

#[derive(PartialEq, Debug)]
pub enum LitKind {
    Str(String),
    Float(f32),
    Integer(i32, /* is_hex */ bool),
}

#[derive(EnumString, PartialEq, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum KwKind {
    As,
    Auto,
    AutoReadOnly,
    Bool,
    Else,
    ElseIf,
    EndEvent,
    EndFunction,
    EndIf,
    EndProperty,
    EndState,
    EndWhile,
    Event,
    Extends,
    False,
    Float,
    Function,
    Global,
    If,
    Import,
    Int,
    Length,
    Native,
    New,
    None,
    Parent,
    Property,
    Return,
    ScriptName,
    #[strum(serialize = "self")]
    _Self,
    State,
    String,
    True,
    While,

    // FO4 keywords
    BetaOnly,
    Const,
    CustomEvent,
    CustomEventName,
    DebugOnly,
    EndGroup,
    EndStruct,
    Group,
    Is,
    ScriptEventName,
    Struct,
    StructVarName,
    Var,
}

#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Eof,
    Whitespace,
    Newline(/* is_crlf */ bool),
    Doc(String),
    Comment(String),
    Literal(LitKind),
    Ident(String),
    Keyword(KwKind),

    LParen,
    RParen,
    LSquare,
    RSquare,
    Dot,
    Comma,

    Minus,
    MinusEq,
    Plus,
    PlusEq,
    Equal,
    Not,
    Multiply,
    MultiplyEq,
    Divide,
    DivideEq,
    Modulo,
    ModuleEq,

    CmpEQ,
    CmpNE,
    CmpLT,
    CmpLE,
    CmpGT,
    CmpGE,
    And,
    Or,
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    lo: usize,
    hi: usize,
}

impl Token {
    fn new(kind: TokenKind, lo: usize, hi: usize) -> Self {
        if lo > hi {
            panic!("Low byte should never be higher than high byte.")
        };
        Token { kind, lo, hi }
    }
}

pub struct Lexer<'a> {
    sess: &'a ParserSession,
    initial_len: usize,
    bytes: Bytes<'a>,
}

impl<'a> Lexer<'a> {
    pub fn from_sess(sess: &'a ParserSession) -> Self {
        let bytes = sess.src.content.bytes();
        Self { initial_len: bytes.len(), sess, bytes }
    }

    fn cur_pos(&self) -> usize {
        self.initial_len - self.bytes.len()
    }

    fn is_eof(&self) -> bool {
        self.bytes.len() == 0
    }

    fn peek_byte(&self) -> u8 {
        self.bytes.clone().nth(0).unwrap_or(EOF_CHAR)
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.bytes.next()
    }

    fn has_equal_next(&mut self, yes: TokenKind, no: TokenKind) -> TokenKind {
        if self.peek_byte() == b'=' {
            self.next_byte(); // auto skip the =
            yes
        } else {
            no
        }
    }

    pub fn next_token(&mut self) -> Token {
        let start_pos = self.cur_pos();
        let first_byte = self.next_byte().unwrap_or(EOF_CHAR);
        let token_kind = match first_byte {
            EOF_CHAR => TokenKind::Eof,
            b if is_whitespace(b) => self.whitespace(),
            b if is_newline_start(b) => self.newline(first_byte),
            b'{' => self.documentation(),
            b';' => match self.peek_byte() {
                b'/' => self.block_comment(),
                _ => self.line_comment(),
            },
            b'"' => TokenKind::Literal(self.string()),
            b @ b'0'..=b'9' => TokenKind::Literal(self.number(b)),
            b if is_id_start(b) => self.ident(b),
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'[' => TokenKind::LSquare,
            b']' => TokenKind::RSquare,
            b'.' => TokenKind::Dot,
            b',' => TokenKind::Comma,
            b'-' => match self.peek_byte() {
                b'=' => {
                    self.next_byte();
                    TokenKind::MinusEq
                }
                b @ b'0'..=b'9' => TokenKind::Literal(self.number(b)),
                _ => TokenKind::Minus,
            },
            b'+' => self.has_equal_next(TokenKind::PlusEq, TokenKind::Plus),
            b'=' => self.has_equal_next(TokenKind::CmpEQ, TokenKind::Equal),
            b'!' => self.has_equal_next(TokenKind::CmpNE, TokenKind::Not),
            b'*' => self.has_equal_next(TokenKind::MultiplyEq, TokenKind::Multiply),
            b'/' => self.has_equal_next(TokenKind::DivideEq, TokenKind::Divide),
            b'%' => self.has_equal_next(TokenKind::ModuleEq, TokenKind::Modulo),
            b'<' => self.has_equal_next(TokenKind::CmpLE, TokenKind::CmpLT),
            b'>' => self.has_equal_next(TokenKind::CmpGE, TokenKind::CmpGT),
            b'&' => match self.peek_byte() {
                b'&' => {
                    self.next_byte();
                    TokenKind::And
                }
                _ => {
                    self.sess
                        .new_error()
                        .warning("expected second '&' for binary AND")
                        .span(start_pos, self.cur_pos())
                        .label_help("try using '&&' instead")
                        .emit();
                    TokenKind::And
                }
            },
            b'|' => match self.peek_byte() {
                b'|' => {
                    self.next_byte();
                    TokenKind::Or
                }
                _ => {
                    self.sess
                        .new_error()
                        .warning("expected second '|' for binary OR")
                        .span(start_pos, self.cur_pos())
                        .label_help("try using '||' instead")
                        .emit();
                    TokenKind::Or
                }
            },
            _ => {
                self.sess
                    .new_error()
                    .fatal("unknown lexeme")
                    .span(start_pos, self.cur_pos())
                    .label_help("are you using unicode characters for an identifier?")
                    .emit();
                unreachable!()
            }
        };
        Token::new(token_kind, start_pos, self.cur_pos())
    }

    fn whitespace(&mut self) -> TokenKind {
        while is_whitespace(self.peek_byte()) {
            self.next_byte();
        }
        TokenKind::Whitespace
    }

    fn newline(&mut self, first_byte: u8) -> TokenKind {
        let is_crlf = first_byte == b'\r';
        if is_crlf && self.peek_byte() == b'\n' {
            self.next_byte();
        }
        TokenKind::Newline(is_crlf)
    }

    fn documentation(&mut self) -> TokenKind {
        let mut value = String::new();
        let mut terminated = false;
        while let Some(b) = self.next_byte() {
            match b {
                b'}' => {
                    terminated = true;
                    break;
                }
                _ => value.push(b as char),
            }
        }
        if !terminated {
            let lo = self.cur_pos() - value.len() - 1;
            let hi = lo + value.find('\n').unwrap_or(0) + 1;
            self.sess.new_error().fatal("unterminated documentation block").span(lo, hi).emit();
            unreachable!()
        }
        TokenKind::Doc(value)
    }

    fn block_comment(&mut self) -> TokenKind {
        let mut value = String::new();
        let mut terminated = false;
        self.next_byte(); // skip the first /
        while let Some(b) = self.next_byte() {
            match b {
                b'/' => {
                    if self.peek_byte() == b';' {
                        terminated = true;
                        self.next_byte();
                        break;
                    }
                    value.push(b as char)
                }
                _ => value.push(b as char),
            }
        }
        if !terminated {
            let lo = self.cur_pos() - value.len() - 2;
            let hi = lo + value.find('\n').unwrap_or(0) + 2;
            self.sess.new_error().fatal("unterminated block comment").span(lo, hi).emit();
            unreachable!()
        }
        TokenKind::Comment(value)
    }

    fn line_comment(&mut self) -> TokenKind {
        let mut value = String::new();
        while !is_newline_start(self.peek_byte()) && !self.is_eof() {
            value.push(self.next_byte().unwrap() as char);
        }
        TokenKind::Comment(value)
    }

    fn string(&mut self) -> LitKind {
        let mut value = String::new();
        let mut terminated = false;
        while let Some(b) = self.next_byte() {
            match b {
                b'"' => {
                    terminated = true;
                    break;
                }
                b if is_newline_start(b) => break,
                b'\\' => {
                    match self.peek_byte() {
                        b'n' => value.push('\n'),
                        b't' => value.push('\t'),
                        b'\\' => value.push('\\'),
                        b'"' => value.push('"'),
                        _ => {
                            self.sess
                                .new_error()
                                .fatal("invalid escape character")
                                .span(self.cur_pos() - 1, self.cur_pos() + 1)
                                .label_error("only '\\n','\\t', '\\\\' or '\\\"' allowed")
                                .emit();
                            unreachable!()
                        }
                    }
                    self.next_byte();
                }
                _ => value.push(b as char),
            }
        }
        if !terminated {
            let lo = self.cur_pos() - value.len() - 2;
            let hi = lo + value.len() + 1;
            self.sess.new_error().fatal("unterminated string").span(lo, hi).emit();
            unreachable!()
        }
        LitKind::Str(value)
    }

    fn number(&mut self, first_digit: u8) -> LitKind {
        let mut value = String::new();
        if first_digit == b'0' && self.peek_byte() == b'x' {
            // hex literal
            self.next_byte(); // skip '0x'
            while !self.is_eof() {
                match self.peek_byte() {
                    b @ b'0'..=b'9' | b @ b'a'..=b'f' | b @ b'A'..=b'F' => value.push(b as char),
                    _ => break,
                };
                self.next_byte();
            }
            if let Ok(lit) = i32::from_str_radix(&value, 16) {
                return LitKind::Integer(lit, true);
            } else {
                let hi = self.cur_pos();
                let lo = hi - value.len() - 2;
                self.sess
                    .new_error()
                    .fatal("could not parse hex literal")
                    .span(lo, hi)
                    .label_error("not a valid hex literal")
                    .emit();
            }
        }
        value.push(first_digit as char);
        let mut is_float = false;
        while !self.is_eof() {
            match self.peek_byte() {
                b @ b'0'..=b'9' => value.push(b as char),
                b'.' => {
                    is_float = true;
                    value.push('.');
                }
                _ => break,
            }
            self.next_byte();
        }
        if is_float {
            if let Ok(lit) = value.parse::<f32>() {
                LitKind::Float(lit)
            } else {
                let hi = self.cur_pos();
                let lo = hi - value.len();
                self.sess.new_error().fatal("could not parse float literal").span(lo, hi).emit();
                unreachable!()
            }
        } else if let Ok(lit) = value.parse::<i32>() {
            LitKind::Integer(lit, false)
        } else {
            let hi = self.cur_pos();
            let lo = hi - value.len();
            self.sess
                .new_error()
                .fatal("could not parse integer literal")
                .span(lo, hi)
                .label_help("try using a smaller integer")
                .emit();
            unreachable!()
        }
    }

    fn ident(&mut self, first_char: u8) -> TokenKind {
        let mut value = String::new();
        value.push(first_char as char);
        while is_id_continue(self.peek_byte(), &self.sess.game) {
            value.push(self.next_byte().unwrap() as char);
        }
        if let Ok(kind) = KwKind::from_str(&value.to_ascii_lowercase()) {
            TokenKind::Keyword(kind)
        } else {
            TokenKind::Ident(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace() {
        let sess = ParserSession::from_string(" \t", Game::TESV);
        let mut lexer = Lexer::from_sess(&sess);
        assert_eq!(Token::new(TokenKind::Whitespace, 0, 2), lexer.next_token());
        assert_eq!(Token::new(TokenKind::Eof, 2, 2), lexer.next_token());
    }

    #[test]
    fn newline() {
        let sess = ParserSession::from_string("\n \r\n", Game::TESV);
        let mut lexer = Lexer::from_sess(&sess);
        assert_eq!(Token::new(TokenKind::Newline(false), 0, 1), lexer.next_token());
        lexer.next_token();
        assert_eq!(Token::new(TokenKind::Newline(true), 2, 4), lexer.next_token());
    }

    #[test]
    fn documentation() {
        let sess = ParserSession::from_string("{ example\ndoc}", Game::TESV);
        let mut lexer = Lexer::from_sess(&sess);
        assert_eq!(
            Token::new(TokenKind::Doc(" example\ndoc".to_string()), 0, 14),
            lexer.next_token()
        );
    }

    #[test]
    fn comment() {
        let sess = ParserSession::from_string(";/ block_comment /; ; line_comment\n ", Game::TESV);
        let mut lexer = Lexer::from_sess(&sess);
        assert_eq!(
            Token::new(TokenKind::Comment(" block_comment ".to_string()), 0, 19),
            lexer.next_token()
        );
        lexer.next_token();
        assert_eq!(
            Token::new(TokenKind::Comment(" line_comment".to_string()), 20, 34),
            lexer.next_token()
        );
    }

    #[test]
    fn ident() {
        let sess = ParserSession::from_string("scriptname identifier", Game::TESV);
        let mut lexer = Lexer::from_sess(&sess);
        assert_eq!(Token::new(TokenKind::Keyword(KwKind::ScriptName), 0, 10), lexer.next_token());
        lexer.next_token();
        assert_eq!(
            Token::new(TokenKind::Ident("identifier".to_string()), 11, 21),
            lexer.next_token()
        );
    }
}
