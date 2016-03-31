//! # Lexer
//!
//! A `Lexer` parses a source string into a list of tokens, which may later be used to construct an
//! Abstract Syntax Tree.
//!
//! ## Notes
//!
//! We want meaningful errors from the start. That means printing the line and column number on
//! error, returning `Result`s instead of panicking (later on, we may use unwinding to speed up
//! lexical analysis in non-erroneous cases).
//!
//! It is unclear whether we should operator on Unicode `char`, or plain bytes `u8`. `char`s are
//! more convenient to display and offer a clean API; bytes are (most likely) faster to work with.

use std::iter::Iterator;

mod token;
pub use self::token::*;

#[cfg(test)]
mod test;

pub struct Lexer<'src> {
    /// Byte offset from the start.
    pos: usize,
    /// The source string.
    src: &'src str,
    /// The last char that was read.
    current_char: Option<char>,
}

impl<'src> Lexer<'src> {
    pub fn new(s: &str) -> Lexer {
        let first_char = s.chars().next();
        println!("first_char: {:?}", first_char);
        let mut l = Lexer {
            src: s,
            pos: 0,
            current_char: first_char, // Ugly?
        };

        l
    }

    /// 'eat' one character.
    fn bump(&mut self) {
        let old = self.current_char;
        self.pos += 1;

        if self.pos < self.src.len() {
            let ch = char_at(&self.src, self.pos);
            self.current_char = Some(ch);
        } else {
            self.current_char = None;
        }
    }

    fn scan_identifier(&mut self) -> String {
        unimplemented!()
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Token;

    /// Return the next token, if any.
    ///
    /// A fundamental property of this function is that **the next token does not depend on the
    /// previous one**.
    /// This means many syntactically incorrect inputs, such as `, , ,` or `;+m/^`, can pass
    /// tokenization, even though they would fail parsing.
    /// This also means testing whether a single token is tokenized properly does not require
    /// scaffolding (i.e. building an entire test program), which is a good thing.
    ///
    /// # Example
    ///
    /// ```
    /// use rgo::lexer::{Lexer, Token, DelimToken};
    ///
    /// let mut lexer = Lexer::new(")");
    /// assert_eq!(lexer.next(), Some(Token::CloseDelim(DelimToken::Paren)));
    /// ```
    fn next(&mut self) -> Option<Token> {
        // Stop tokenizing on EOF.
        let c = match self.current_char {
            Some(c) => c,
            None => return None,
        };

        let tok = match c {
            // Single-character tokens.
            '(' => {
                self.bump();
                Token::OpenDelim(DelimToken::Paren)
            }
            ')' => {
                self.bump();
                Token::CloseDelim(DelimToken::Paren)
            }
            '{' => {
                self.bump();
                Token::OpenDelim(DelimToken::Brace)
            }
            '}' => {
                self.bump();
                Token::CloseDelim(DelimToken::Brace)
            }
            '[' => {
                self.bump();
                Token::OpenDelim(DelimToken::Bracket)
            }
            ']' => {
                self.bump();
                Token::CloseDelim(DelimToken::Bracket)
            }
            ',' => {
                self.bump();
                Token::Comma
            }
            // More complex tokens.
            '.' => {
                self.bump();
                // FIXME: ellipsis '...'
                Token::Dot
            }
            '+' => {
                self.bump();

                match self.current_char {
                    Some('+') => {
                        self.bump();
                        Token::Increment
                    }
                    Some('=') => {
                        self.bump();
                        Token::PlusEquals
                    }
                    _ => Token::Plus,
                }
            }
            '-' => {
                self.bump();

                match self.current_char {
                    Some('-') => {
                        self.bump();
                        Token::Decrement
                    }
                    Some('=') => {
                        self.bump();
                        Token::MinusEquals
                    }
                    _ => Token::Minus,
                }
            }
            '|' => {
                self.bump();

                match self.current_char {
                    Some('|') => {
                        self.bump();
                        Token::PipePipe
                    }
                    Some('=') => {
                        self.bump();
                        Token::PipeEquals
                    }
                    _ => Token::Pipe,
                }
            }
            c if can_start_identifier(c) => {
                let start = self.pos;
                println!("c: {}", c);

                loop {
                    if let Some(c) = self.current_char {
                        println!("ident c: {}", c);
                        if can_continue_identifier(c) {
                            self.bump();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                let ident = &self.src[start..self.pos];

                match &*ident {
                    "break" => Token::Keyword(Keyword::Break),
                    "case" => Token::Keyword(Keyword::Case),
                    "chan" => Token::Keyword(Keyword::Chan),
                    "const" => Token::Keyword(Keyword::Const),
                    "continue" => Token::Keyword(Keyword::Continue),
                    "default" => Token::Keyword(Keyword::Default),
                    "defer" => Token::Keyword(Keyword::Defer),
                    "else" => Token::Keyword(Keyword::Else),
                    "fallthrough" => Token::Keyword(Keyword::Fallthrough),
                    "for" => Token::Keyword(Keyword::For),
                    "func" => Token::Keyword(Keyword::Func),
                    "go" => Token::Keyword(Keyword::Go),
                    "goto" => Token::Keyword(Keyword::Goto),
                    "if" => Token::Keyword(Keyword::If),
                    "import" => Token::Keyword(Keyword::Import),
                    "interface" => Token::Keyword(Keyword::Interface),
                    "map" => Token::Keyword(Keyword::Map),
                    "package" => Token::Keyword(Keyword::Package),
                    "range" => Token::Keyword(Keyword::Range),
                    "return" => Token::Keyword(Keyword::Return),
                    "select" => Token::Keyword(Keyword::Select),
                    "struct" => Token::Keyword(Keyword::Struct),
                    "switch" => Token::Keyword(Keyword::Switch),
                    "type" => Token::Keyword(Keyword::Type),
                    "var" => Token::Keyword(Keyword::Var),

                    // `ident` is not a keyword.
                    // XXX(perf): unnecessary alloc.
                    _ => Token::Ident(ident.into()),
                }
            }
            c if c.is_whitespace() => {
                println!("ws start c: {}", c);
                // XXX: this loop pattern is not pretty.
                loop {
                    if let Some(c) = self.current_char {
                        println!("ws c: {}", c);
                        if c.is_whitespace() {
                            self.bump();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Token::Whitespace

            }
            '"' => {
                self.bump();
                let start = self.pos;

                while let Some(c) = self.current_char {
                    // FIXME: backslash
                    if c != '"' {
                        self.bump();
                    } else {
                        break;
                    }
                }

                let s = &self.src[start..self.pos];

                // Skip the quote.
                self.bump();
                // XXX(perf): alloc.
                Token::Literal(Literal::Str(s.into()))
            }
            c => panic!("unexpected start of token: '{}'", c),
        };

        Some(tok)
    }
}

/// Convenience function to collect all the tokens from a string.
///
/// # Example
///
/// ```
/// use rgo::lexer::{tokenize, Token, DelimToken};
///
/// assert_eq!(tokenize("()"), vec![
///     Token::OpenDelim(DelimToken::Paren),
///     Token::CloseDelim(DelimToken::Paren)
/// ]);
/// ```
pub fn tokenize(s: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(s);
    let tokens: Vec<Token> = lexer.collect();

    tokens
}


// Unicode Scalar Value = Any Unicode code point except high-surrogate and low-surrogate code
// points.

// XXX(perf): expensive check on Unicode chars.

fn can_start_identifier(c: char) -> bool {
    c.is_alphabetic()
}

fn can_continue_identifier(c: char) -> bool {
    c.is_alphabetic() || c.is_numeric()
}

pub fn char_at(s: &str, byte: usize) -> char {
    s[byte..].chars().next().unwrap()
}
