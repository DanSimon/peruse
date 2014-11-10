#![feature(globs)]
#![feature(phase)]
#![feature(unboxed_closures)]
#![feature(macro_rules)]

extern crate regex;


pub mod parsers;

#[macro_escape]
pub mod macros;

mod tests;

