#Peruse

Peruse is a small parser-combinator library for rust.  The goal is to be able
to write clean, efficient parsers powerful enough to handle most grammars.

This project is my first foray into rust and is very much a work-in-progress.
Comments, suggestions, and PR's are welcome.

For a sorta real-world example, checkout
[Coki](https://github.com/DanSimon/coki) a small programming language that uses
Peruse for both its lexer and parser.

The tests have some basic examples of how to use the combinators.  You don't
have to use the macros, but without them it gets pretty ugly.

## Building

I am building against the Rust nightlies until 1.0 hits.

When importing the crate, be sure to use the `#[phase(Plugin)]` attribute, eg

```
#[phase(plugin)] extern crate peruse;
```
to bring the macros into scope

