use std::rc::Rc;
use parsers::{Parser, ParserCombinator, ParseResult};

pub type SliceParser<I,O> = Parser<I=[I], O=O>;

/// Create a parser that only recognizes the given literal value
pub fn lit<T: Eq + Clone>(l: T) -> LiteralParser<T> {
  LiteralParser{literal: l}
}

pub fn matcher<T: Clone, U, F: 'static + Fn(T) -> Option<U>>(f: F) -> MatchParser<T, U> {
  MatchParser{matcher: Rc::new(Box::new(f))}
}



//////////////////////// STRUCTS /////////////////////////////////////////////



/// A LiteralParser looks for an exact match of the given item at the beginning
// of the slice
#[derive(Clone)]
pub struct LiteralParser< T: Eq + Clone> {
  pub literal: T,
}

impl<T: Eq + Clone> Parser for LiteralParser< T> {
  type I = [T];
  type O = T;

  fn parse<'a>(&self, data: &'a [T]) -> ParseResult<&'a [T], T> {
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

impl<T: Eq + Clone> ParserCombinator for LiteralParser<T>{}



pub struct MatchParser<T: Clone, U> {
  matcher: Rc<Box<Fn(T) -> Option<U>>>
}

impl<T: Clone, U> Parser for MatchParser<T,U> {
  type I = [T];
  type O = U;

  fn parse<'a>(&self, data: &'a [T]) -> ParseResult<&'a [T], Self::O> {
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

  
