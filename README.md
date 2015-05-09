#Peruse

Peruse is a small parser-combinator library for rust.  The goal is to be able
to write clean, efficient parsers powerful enough to handle most grammars.
This project is my first foray into rust and is very much a work-in-progress.
Comments, suggestions, and PR's are welcome.

A parser is an object that translates some input type into an output type.
While these parsers can work with any input and output types, they're mostly
focused on turning a sequence of items into a more structured form such as an
abstract syntax tree.  Every parser returns an output value along with another
input value, which for slices is the remaining portion of the input.  Thus
parsers can be chained together, so that the result from one parser is fed into
the next.

Here's a quick example of how simple parsers are used to build more complex parsers

```rust
use slice_parsers::*;

//let's say we have the following array
let arr = [1, 0, 1, 0, 1, 0];

//how about we write a parser to count the number of sequences of 1, 0

let p1 = lit(1).then(lit(0)).repeat().map(|v| v.length());

//calling parse will return a ParseResult, containing the parsed value along
//with a slice of any unparsed data
let res = p2.parse(&arr); //Ok(3, [])

//define some input tokens
enum Token { A, B, C }

//and a final result type
struct Foo{num: uint, has_c: bool}

//start with a parser to consume one element of a slice of tokens if it matches
//the given token
let lit_parser = literal(A);
let data1 = [A, B, C];
//the parser returns the literal along with a reference to the remaining input data
let result = lit_parser.parse(&data1) //Ok( (A, &[B, C]) )

//now lets create a parser to handle A or B
let or_parser = or!(literal(A), literal(B))
or_parser.parse(&[A]); //Ok
or_parser.parse(&[B]); //Ok
or_parser.parse(&[C]); //Err

//now we can repeat the parser until it fails
let repeat_parser = rep!(or_parser)
let data2 = [A, B, B, A, C];
repeat_parser.parse(&data2); //Ok( (vec![A, B, B, A], &[C]) )

//and we can sequence it with another parser 
let sequenced_parser = seq!(repeat_parser, opt!(literal(C)));

//lastly we can map the results from the parser to some other type
let mapped = map!(sequenced_parser, |&: (vec, opt)| Foo{num: vec.len(), has_c: opt.is_some()});

mapped.parse(&data2) // Ok( (Foo{num: 4, has_c: true} , &[]) )
```

The parsers themselves are just some boxed structs implementing a trait, but it's much easier to use the inluded macros.  So far we have:

* **seq!(a, b, ...)** - chain together some parsers, putting their results in a tuple.
* **or!(a, b, ...)**  - use parser `a`, if it fails then try `b`, and so on.
* **opt!(a)** - lift a parser into an `Option`, returning `Some` if it succeeds and `None` otherwise.
* **rep!(a)** - repeat `a` until it fails, accumulating the results into a vector
* **repsep!(rep, sep)** - repeatedly parse `rep` and then `sep` until `sep` fails, accumulating the results from `rep` into a vector
* **map!(p, closure)**  - map the result from p using an unboxed closure

## Recursive Grammars

Implementing a recursive grammar currently requires a little more overhead.
The only real way to implement a recursive parser is to wrap it in a function.
And since Rust does not allow type inference on function return values, the
only sane way to write such functions is to box the result so the return type
is just `Parser<'a, Foo, Bar>` and not `OrParser<'a, OrParser<'a, OrParser<'a,
DualParser<'a, Foo, Bar.....` (of course, you can try avoiding the boxing, but the types get really gnarly).

The `lazy!` macro can wrap such a function.  Here's an example implementing addition expressions with parentheses (let's pretend addition isn't associative):

```rust
enum Token {
  OpenParen,
  CloseParen,
  PlusSign,
  Number(uint),
}

enum AST {
  Plus(AST, AST),
  Addend(uint),
}

fn expr<'a>() -> Parser<'a, Token, AST> {
  box or!(
    matcher!(Token: Number(i) => Addend(i)),
    seq!(literal(OpenParen), lazy!(expr), literal(CloseParen) to |&: (_, (expr, _))| expr),
    seq!(lazy!(expr()), PlusSign, lazy!(expr()) to |&: (lhs, (_, rhs))| Plus(lhs, rhs)),
  )
}
```

### More Examples

The [tests](src/peruse/tests.rs) have some basic examples of how to use the combinators.  You don't
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

