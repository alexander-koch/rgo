//! Welcome to `rgo`!
//!
//! Compilation is a pipeline, which works (roughly) like this:
//!
//! Lexical analysis -> Parsing -> Translation
//!
//! Each part should be loosely coupled with the others, only relying on a somewhat stable public
//! API.

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate log;
// XXX: do we still need it?
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;

mod pos;
pub use self::pos::Position;

pub mod token;
pub mod ast;
pub mod lexer;
pub mod parser;

pub use parser::Parser;

pub fn parse(src: &str) -> ast::SourceFile {
    let lexer = lexer::Lexer::new(src).collect();
    parser::parse_tokens(lexer)
}
