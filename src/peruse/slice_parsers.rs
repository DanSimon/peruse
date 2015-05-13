use core::marker::PhantomData;
use std::rc::Rc;
use std::str;

/// A Parser<'a> is a parser that parses some elements out of the beginning of
/// a slice and returns a parsed value along with the rest of the unparsed slice
pub trait Parser<'a>  {
  type I;
  type O;

  fn parse(&self, data: Self::I) -> ParseResult<Self::I, Self::O>;
}

/// Combinator methods for slice parsers.  In most cases, these methods copy
/// the caller into a higher-order parser
pub trait ParserCombinator<'a> : Parser<'a> + Clone {

  /// Chain this parser with another parser, creating new parser that returns a
  /// tuple of their results
  fn then<P: Parser<'a, I=Self::I>>(&self, p: P) -> ChainedParser<Self,P> {
    ChainedParser::new(self.clone(), p)
  }

  /// Create a new parser that will repeat this parser until it returns an error
  fn repeat(&self) -> RepeatParser<Self> {
    RepeatParser{parser: self.clone()}
  }

  /// Chain this parser with another parser, but toss the value from this parser
  fn then_r<P: ParserCombinator<'a, I=Self::I>>(&self, p: P) -> MapParser<ChainedParser<Self, P>, (Self::O, P::O), P::O> {
    self.then(p).map(|(_, t)| t)
  }

  /// Chain this parser with another parser, but toss the value from the other parser
  fn then_l<P: ParserCombinator<'a, I=Self::I>>(&self, p: P) -> MapParser<ChainedParser<Self, P>, (Self::O, P::O), Self::O> {
    self.then(p).map(|(t, _)| t)
  }

  
  /// Map the value of this parser
  fn map<T, F: 'static + Fn(Self::O) -> T>(&self, f: F) -> MapParser<Self, Self::O, T> {
    MapParser{parser: self.clone(), mapper: Rc::new(Box::new(f))}
  }

/*
  /// Create a disjunction with another parser.  If this parser produces an error, the other parser will be used
  fn or<P: Parser<'a><I=Self::I, O=Self::O>>(&self, p: P) -> OrParser<Self::I, Self::O, Self,P> {
    OrParser{first: self.clone(), second: p}
  }
  */

}

pub type ParseResult<I,O> = Result<(O, I), String>;


/// Create a parser that only recognizes the given literal value
pub fn lit<'a, T: 'a + Eq + Clone>(l: T) -> LiteralParser<'a, T> {
  LiteralParser::new(l)
}

/*
/// Create a parser that will return Some if the given parser is successful, None otherwise
pub fn opt<T: Parser<'a>>(t: T) -> OptionParser<T> {
  OptionParser{parser: t}
}

/// Create a lazily evaluated parser from a function.  This can be used to generate recursive parsers
pub fn recursive<I,O, F:  Fn() -> Box<Parser<'a><I=I,O=O>>>(f: F) -> RecursiveParser<I,O,F> {
  RecursiveParser{parser: Rc::new(f)}
}

pub fn matcher<T: Clone, U, F: 'static + Fn(T) -> Option<U>>(f: F) -> MatchParser<T, U> {
  MatchParser{matcher: Rc::new(Box::new(f))}
}

*/
//////////////////////// STRUCTS /////////////////////////////////////////////

/// A Chained parser contains two parsers that will be used in sequence to
/// create a tuple of parsed values
pub struct ChainedParser<A,B> {
  first: A,
  second: B,
}

impl<A,B> ChainedParser<A, B> {
  
  pub fn new(first: A, second: B) -> Self {
    ChainedParser{first: first, second: second}
  }
}

impl<'a, C, A: Parser<'a, I=C>, B: Parser<'a, I=C>> Parser<'a> for ChainedParser<A, B> {
  type I = C;
  type O = (A::O,B::O);

  fn parse(&self, data: Self::I) -> ParseResult<Self::I, Self::O>{
    match self.first.parse(data) {
      Ok((a, d2)) => match self.second.parse(d2) {
        Ok((b, remain)) => Ok(((a, b), remain)),
        Err(err) => Err(err)
      },
      Err(err) => Err(err)
    }
  }
}

impl<A: Clone, B:Clone>  Clone for ChainedParser<A, B> {
  
  fn clone(&self) -> Self {
    ChainedParser{first: self.first.clone(), second: self.second.clone()}
  }
}

impl<'a, C, A: ParserCombinator<'a, I=C>, B: ParserCombinator<'a, I=C>>  ParserCombinator<'a> for ChainedParser<A, B> {}


/// A LiteralParser looks for an exact match of the given item at the beginning
// of the slice
#[derive(Clone)]
pub struct LiteralParser<'a,  T: 'a + Eq + Clone> {
  pub literal: T,
  _marker: PhantomData<&'a T>
}

impl<'a, T: 'a + Eq + Clone> LiteralParser<'a, T> {
  pub fn new(l: T) -> Self {
    LiteralParser{literal: l, _marker: PhantomData}
  }
}

impl<'a, T: 'a + Eq + Clone> Parser<'a> for LiteralParser<'a, T> {
  type I = &'a [T];
  type O = T;

  fn parse(&self, data: &'a [T]) -> ParseResult<&'a [T], T> {
    if data.len() < 1 {
      return Err(format!("ran out of data"))
    }
    if data[0] == self.literal {
      Ok((data[0].clone(), &data[1..]))
    } else {
      Err(format!("Literal mismatch"))
    }
  }
}

impl<'a, T: 'a + Eq + Clone> ParserCombinator<'a> for LiteralParser<'a, T>{}


#[derive(Clone)]
pub struct StringLiteralParser{
  lit: &'static str,
}

impl<'a> Parser<'a> for StringLiteralParser {
  type I = &'a str;
  type O = &'a str;

  fn parse(&self, data: Self::I) -> ParseResult<Self::I, Self::O>{
    let b = self.lit.as_bytes();
    let d = data.as_bytes();
    if d.starts_with(b) {
      let l: &'a str = unsafe {
        str::from_utf8_unchecked(&d[0..b.len()])
      };
      let r: &'a str = unsafe {
        str::from_utf8_unchecked(&d[b.len()..])
      };
      Ok((l, r))
    } else {
      Err(format!("expected {}", self.lit))
    }
  }
}

impl<'a> ParserCombinator<'a> for StringLiteralParser {}

/// A Parser that repeats the given parser until it encounters an error.  A
/// vector of the accumulated parsed values is returned
pub struct RepeatParser<P> {
  parser: P,
}
impl<'a, U: Clone + Sized, T: Parser<'a, I=U>> Parser<'a> for RepeatParser<T> {
  type I = T::I;
  type O = Vec<T::O>;
  
  fn parse(&self, data: Self::I) -> ParseResult<Self::I, Self::O> {
    let mut remain = data;
    let mut v: Vec<T::O> = Vec::new();
    loop {
      match self.parser.parse(remain.clone()) {
        Ok((result, rest)) => {
          v.push(result);
          remain = rest;
        }
        Err(_) => {
          return Ok((v, remain));
        }
      }
    }
  }
}

impl<'a, U: Clone + Sized, T: ParserCombinator<'a,I=U>> ParserCombinator<'a> for RepeatParser<T> {}

impl<'a, T: ParserCombinator<'a>> Clone for RepeatParser<T> {
  fn clone(&self) -> Self {
    RepeatParser{parser: self.parser.clone()}
  }
}

/// A Parser that uses a closure to map the result of another parser
pub struct MapParser<P, I, T> {
  parser: P,
  mapper: Rc<Box<Fn(I) -> T>>,
}

impl<'a, I, P: Parser<'a, O=I>, T> Parser<'a> for MapParser<P,I,T> {
  type I = P::I;
  type O = T;

  fn parse(&self, data: Self::I) -> ParseResult<Self::I, Self::O> {
    self.parser.parse(data).map(|(output, input)| ((self.mapper)(output), input))
  }

}


impl<'a, P: ParserCombinator<'a>, T> Clone for MapParser<P,P::O, T> {

  fn clone(&self) -> Self {
    MapParser{parser: self.parser.clone(), mapper: self.mapper.clone()}
  }
}


impl<'a, I, P: ParserCombinator<'a, O=I>, T> ParserCombinator<'a> for MapParser<P,I,T>  {}

/*

pub struct OrParser<I,O, S: Parser<'a><I=I,O=O>, T: SliceParser<I=I,O=O>> {
  first: S,
  second: T,
}

impl<I,O, S: Parser<'a><I=I,O=O>, T: SliceParser<I=I,O=O>> SliceParser for OrParser<I,O,S,T> {
  type I = I;
  type O = O;

  fn parse(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
    match self.first.parse(data.clone()) {
      Ok((a, d2)) => Ok((a, d2)),
      Err(_) => match self.second.parse(data.clone()) {
        Ok((b, remain)) => Ok((b, remain)),
        Err(err) => Err(err)
      }
    }
  }
}

impl<I,O, S: ParserCombinator<I=I,O=O>, T: ParserCombinator<I=I,O=O>> Clone for OrParser<I,O,S,T> {

  fn clone(&self) -> Self {
    OrParser{first: self.first.clone(), second: self.second.clone()}
  }
}

impl<I,O, S: ParserCombinator<I=I,O=O>, T: ParserCombinator<I=I,O=O>> ParserCombinator for OrParser<I,O,S,T> {}


#[derive(Clone)]
pub struct OptionParser<P: Parser<'a>> {
  parser: P 
}
impl<P: Parser<'a>> SliceParser for OptionParser<P> {
  type I = P::I;
  type O = Option<P::O>;

  fn parse(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
    match self.parser.parse(data.clone()) {
      Ok((result, rest))  => Ok((Some(result), rest)),
      Err(_)              => Ok((None, data)),
    }
  }
}

pub struct RecursiveParser<I, O, F> where F: Fn() -> Box<Parser<'a><I=I,O=O>>{
  parser: Rc<F>
}

impl<I, O, F> Parser<'a> for RecursiveParser<I, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>> {

  type I = I;
  type O = O;

  fn parse(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
    (self.parser)().parse(data)
  }

}

impl<I, O, F> ParserCombinator for RecursiveParser<I, O, F> where F: Fn() -> Box<Parser<'a><I=I,O=O>> {}

impl<I, O, F> Clone for RecursiveParser<I, O, F> where F: Fn() -> Box<Parser<'a><I=I,O=O>> {
  fn clone(&self) -> Self {
    RecursiveParser{parser: self.parser.clone()}
  }
}


pub struct MatchParser<T: Clone, U> {
  matcher: Rc<Box<Fn(T) -> Option<U>>>
}

impl<'a, T: Clone, U> Parser<'a> for MatchParser<T,U> {
  type I = T;
  type O = U;

  fn parse(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
    if data.len() < 1 {
      return Err(format!("ran out of data"))
    }
    match (self.matcher)(data[0].clone()) {
      Some(u) => Ok((u, &data[1..])),
      None    => Err(format!("Match failed"))
    }
  }
}


impl<T: Clone, U> ParserCombinator for MatchParser<T,U> {}

impl<T: Clone, U> Clone for MatchParser<T,U> {

  fn clone(&self) -> Self {
    MatchParser{matcher: self.matcher.clone()}
  }

}

*/
