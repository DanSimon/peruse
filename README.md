#Peruse

Peruse is a small parser-combinator library for rust.  The goal is to be able
to write clean, efficient parsers powerful enough to handle most grammars.
This project is my first foray into rust and is very much a work-in-progress.
Comments, suggestions, and PR's are welcome.

I initially tried to write these without boxing everything, but it didn't work
out too well, mostly because of the lazy-evaluated `OrParser`.  But the input
data itself is (usually) never boxed or copied, so I don't think it's a big
deal for now.

The [tests](src/tests.rs) have some basic examples of how to use the combinators.  You don't
have to use the macros, but without them it gets pretty ugly.

For a sorta real-world example, check out
[Coki](https://github.com/DanSimon/coki), a small programming language I'm
working on that uses Peruse for both its lexer and parser.


## Known Issues

* sequences of >2 parsers using `seq!` get turned into nested tuples.  Currently trying to figure out if I can write a macro to flatten them.
* Ideally we should remove the `map!` macro and allow all the other macros to take a closure as a final argument, should greatly improve readability in complex parsers.

## Building

I am building against the Rust nightlies until 1.0 hits.

When importing the crate, be sure to use the `#[phase(plugin)]` attribute, eg

```
#[phase(plugin)] extern crate peruse;
```
to bring the macros into scope.

