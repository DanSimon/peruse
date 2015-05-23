#Peruse

[![Build Status](https://travis-ci.org/DanSimon/peruse.svg?branch=master)](https://travis-ci.org/DanSimon/peruse)

[![Crates.io](https://img.shields.io/crates/v/peruse.svg)](https://crates.io/crates/peruse)

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

Peruse contains 2 types of parsers

* Recursive-Descent Parsers - These are your more typical parsers generally used for building recursive data structures like AST's or JSON.

* Stream parsers (coming soon) - These are stateful parsers that are able to receive the input data in pieces.  These are useful mostly for network protocols.


## Examples

### Slice Parsers

A slice parser expects as input a slice of some type T.  Parsers consume one or
more elements at the beginnng of the slice and return an output value along
with the rest of the slice.

```rust
use peruse::*;
use peruse::slice_parsers::*;

//let's start with something simple, a parser that looks for one particular
//integer as the first element of a given slice

let p1 = lit(3);

//calling parse will return a ParseResult, containing the parsed value along
//with a slice of any unparsed data

println!("{:?}", p1.parse(&[3, 1, 2]) );
//Ok((3, [1, 2]))

println!("{:?}", p1.parse(&[4, 1, 2]) );
//Err("Literal mismatch")

//now we can start to chain parsers together

let p2 = lit(3).or(lit(4));

println!("{:?}", p2.parse(&[4, 1, 2]) );
//Ok((4, [1, 2]))

//and turn the parsed items into other types

let p3 = lit(3).or(lit(4)).then(lit(1)).map(|(a, b)| a + b);

println!("{:?}", p3.parse(&[4, 1, 2]) );
//Ok((5, [2]))


//let's say we have the following array
let arr = [1, 0, 1, 0, 1, 0];

//how about we write a parser to count the number of sequences of 1, 0

let p4 = lit(1).then(lit(0)).repeat().map(|v| v.len());

println!("{:?}", p4.parse(&arr)); 
//Ok((3, []))

//lastly we can define a recursive parser in a static function
fn recurse() -> Box<SliceParser<i32, i32>> {
  let end = lit(1).map(|_| 0);
  let rec = lit(0).then_r(recursive(|| recurse())).map(|t| t + 1);
  Box::new(end.or(rec))
}

println!("{:?}",count_zeros().parse(&[0,0,0,0,0,1]));
//Ok((5, []))
```

The included
[tests](https://github.com/DanSimon/peruse/blob/master/src/peruse/slice_parser_tests.rs)
give basic examples of all the existing parsers as well as some more
complicated examples.


For a more real-world example, checkout
[Coki](https://github.com/DanSimon/coki), a very simple programming language
I'm working on.  Peruse is used for both the lexer and AST parser.

## Other Notes

In most cases, constructed parsers use static dispath whenever possible.  My
end goal is static dispatch everywhere, still working on it.

Be aware, due to an ongoing [issue with
rustc](https://github.com/rust-lang/rust/issues/22204), the compile time of
your code will exponentially increase with the complexity of your parsers.  In
practice I've found things get bad after about 10 combinations or so.  You can
get around this by boxing a parser:

```rust
let parser = lit(1).or(lit(2)).or(lit(3)).repeat().then(opt(lit(4).then(lit(5))));
let boxed = boxed(parser);  //creates a BoxedParser
let full_parser = boxed.or(lit(3));
```

This "flattens" the type signature of the parser into a trait object, which
will improve compile-time at the cost of runtime performance due to dynamic
dispath.  But in most cases since you're only doing this on like 1/10th of your
parsing, the performance hit shouldn't be that bad (in theory, I haven't tested
any of this yet).


Right now slice parsers cannot return pointers to the input data.  Trying to figure
out if this will be possible but I think we'll need to wait for higher-kinded
types.  The soon-to-be implemented StreamParsers may allow for this.
