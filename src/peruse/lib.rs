#![feature(unboxed_closures)]
#![allow(unstable)]

extern crate regex;


pub mod parsers;

#[macro_use]
pub mod macros;

mod tests;

mod peruse {
    pub use parsers;
}

