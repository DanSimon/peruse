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

Peruse contains 3 types of parsers

* Slice Parsers - parse slices into values
* String parsers (coming soon) - parse strings
* Stream parsers (coming soon) - parse slices of bytes where a single slice may only contain part of a complete sequence.


## Slice Parsers

Slice parsers take in as input a slice `&[T]`.  This kind of parser is ideal for parsing recursive syntax trees.

```rust
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
fn recurse() -> Box<SliceParser<I=i32, O=i32>> {
  let end = lit(1).map(|_| 0);
  let rec = lit(0).then_r(recursive(|| recurse())).map(|t| t + 1);
  Box::new(end.or(rec))
}

println!("{:?}",recurse().parse(&[0,0,0,0,0,1]));
//Ok((5, []))
```

## Building

I am building against the Rust nightlies until 1.0 hits.

