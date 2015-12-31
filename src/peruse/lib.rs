//! Peruse is a basic parser combinator library for Rust.  
//!
//! Currently the project is split into several modules
//! * parsers - contains the main combinators
//! * slice_parsers - a few simple primitive parsers for handling slices of items
//! * string_parsers - a few parsers for handling strings
//!
//! Parsers work by essentially building a heirarchy of structs that all implement the `Parser` and
//! `Combinator` traits.  



extern crate regex;

pub mod parsers;
pub mod slice_parsers;
pub mod string_parsers;

mod slice_parser_tests;
mod string_parser_tests;
//mod tests;


