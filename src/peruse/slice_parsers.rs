use std::rc::Rc;

/// A SliceParser is a parser that parses some elements out of the beginning of
/// a slice and returns a parsed value along with the rest of the unparsed slice
pub trait SliceParser  {
  type I: ?Sized;
  type O;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O>;
}

/// Combinator methods for slice parsers.  In most cases, these methods copy
/// the caller into a higher-order parser
pub trait ParserCombinator : SliceParser + Clone {

  /// Chain this parser with another parser, creating new parser that returns a
  /// tuple of their results
  fn then<P: SliceParser<I=Self::I>>(&self, p: P) -> ChainedParser<Self,P> {
    ChainedParser{first: self.clone(), second: p}
  }

  /// Chain this parser with another parser, but toss the value from this parser
  fn then_r<P: ParserCombinator<I=Self::I>>(&self, p: P) -> MapParser<Self::I, ChainedParser<Self, P>, P::O> {
    self.then(p).map(|(_, t)| t)
  }

  /// Chain this parser with another parser, but toss the value from the other parser
  fn then_l<P: ParserCombinator<I=Self::I>>(&self, p: P) -> MapParser<Self::I, ChainedParser<Self, P>, Self::O> {
    self.then(p).map(|(t, _)| t)
  }

  /// Create a new parser that will repeat this parser until it returns an error
  fn repeat(&self) -> RepeatParser<Self> {
    RepeatParser{parser: self.clone()}
  }
  
  /// Map the value of this parser
  fn map<T, F: 'static + Fn(Self::O) -> T>(&self, f: F) -> MapParser<Self::I, Self, T> {
    MapParser{parser: self.clone(), mapper: Rc::new(Box::new(f))}
  }

  /// Create a disjunction with another parser.  If this parser produces an error, the other parser will be used
  fn or<P: SliceParser<I=Self::I, O=Self::O>>(&self, p: P) -> OrParser<Self,P> {
    OrParser{first: self.clone(), second: p}
  }

}

pub type ParseResult<I,O> = Result<(O, I), String>;

/// Create a parser that only recognizes the given literal value
pub fn lit<T: Eq + Clone>(l: T) -> LiteralParser<T> {
  LiteralParser{literal: l}
}

/// Create a parser that will return Some if the given parser is successful, None otherwise
pub fn opt<T: SliceParser>(t: T) -> OptionParser<T> {
  OptionParser{parser: t}
}

/// Create a lazily evaluated parser from a function.  This can be used to generate recursive parsers
pub fn recursive<I:?Sized,O, F:  Fn() -> Box<SliceParser<I=I,O=O>>>(f: F) -> RecursiveParser<I,O,F> {
  RecursiveParser{parser: Rc::new(f)}
}

pub fn matcher<T: Clone, U, F: 'static + Fn(T) -> Option<U>>(f: F) -> MatchParser<T, U> {
  MatchParser{matcher: Rc::new(Box::new(f))}
}

pub fn repsep<I: ?Sized, A: SliceParser<I=I>, B: SliceParser<I=I>>(rep: A, sep: B) -> RepSepParser<A,B> {
  RepSepParser{rep: rep, sep: sep, min_reps: 1}
}


//////////////////////// STRUCTS /////////////////////////////////////////////

/// A Chained parser contains two parsers that will be used in sequence to
/// create a tuple of parsed values
pub struct ChainedParser<A,B> {
  first: A,
  second: B,
}
impl<C: ?Sized, A: SliceParser<I=C>, B: SliceParser<I=C>> SliceParser for ChainedParser<A, B> {
  type I = C;
  type O = (A::O,B::O);

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O>{
    match self.first.parse(data) {
      Ok((a, d2)) => match self.second.parse(d2) {
        Ok((b, remain)) => Ok(((a, b), remain)),
        Err(err) => Err(err)
      },
      Err(err) => Err(err)
    }
  }
}

impl<C: ?Sized, A: ParserCombinator<I=C>, B: ParserCombinator<I=C>>  Clone for ChainedParser<A, B> {
  
  fn clone(&self) -> Self {
    ChainedParser{first: self.first.clone(), second: self.second.clone()}
  }
}

impl<C: ?Sized, A: ParserCombinator<I=C>, B: ParserCombinator<I=C>>  ParserCombinator for ChainedParser<A, B> {}


/// A LiteralParser looks for an exact match of the given item at the beginning
// of the slice
#[derive(Clone)]
pub struct LiteralParser< T: Eq + Clone> {
  pub literal: T,
}

impl<T: Eq + Clone> SliceParser for LiteralParser< T> {
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

/// A Parser that repeats the given parser until it encounters an error.  A
/// vector of the accumulated parsed values is returned
pub struct RepeatParser<P: SliceParser> {
  parser: P
}
impl<T: SliceParser> SliceParser for RepeatParser<T> {
  type I = T::I;
  type O = Vec<T::O>;
  
  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
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

impl<T: ParserCombinator> ParserCombinator for RepeatParser<T> {}

impl<T: ParserCombinator> Clone for RepeatParser<T> {
  fn clone(&self) -> Self {
    RepeatParser{parser: self.parser.clone()}
  }
}


/// A Parser that uses a closure to map the result of another parser
pub struct MapParser<I: ?Sized, P: SliceParser<I=I>, T> {
  parser: P,
  mapper: Rc<Box<Fn(P::O) -> T>>,
}

impl<I: ?Sized, P: SliceParser<I=I>, T> SliceParser for MapParser<I,P,T> {
  type I = P::I;
  type O = T;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    self.parser.parse(data).map(|(output, input)| ((self.mapper)(output), input))
  }

}

impl<I: ?Sized, P: ParserCombinator<I=I>, T> Clone for MapParser<I,P,T> {

  fn clone(&self) -> Self {
    MapParser{parser: self.parser.clone(), mapper: self.mapper.clone()}
  }
}

impl<I: ?Sized, P: ParserCombinator<I=I>, T> ParserCombinator for MapParser<I,P,T> {}

pub struct OrParser<S,T> {
  first: S,
  second: T,
}

impl<I:?Sized,O, S: SliceParser<I=I,O=O>, T: SliceParser<I=I,O=O>> SliceParser for OrParser<S,T> {
  type I = I;
  type O = O;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    match self.first.parse(data.clone()) {
      Ok((a, d2)) => Ok((a, d2)),
      Err(_) => match self.second.parse(data.clone()) {
        Ok((b, remain)) => Ok((b, remain)),
        Err(err) => Err(err)
      }
    }
  }
}

impl<I:?Sized,O, S: ParserCombinator<I=I,O=O>, T: ParserCombinator<I=I,O=O>> Clone for OrParser<S,T> {

  fn clone(&self) -> Self {
    OrParser{first: self.first.clone(), second: self.second.clone()}
  }
}

impl<I:?Sized,O, S: ParserCombinator<I=I,O=O>, T: ParserCombinator<I=I,O=O>> ParserCombinator for OrParser<S,T> {}


#[derive(Clone)]
pub struct OptionParser<P: SliceParser> {
  parser: P 
}
impl<P: SliceParser> SliceParser for OptionParser<P> {
  type I = P::I;
  type O = Option<P::O>;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    match self.parser.parse(data.clone()) {
      Ok((result, rest))  => Ok((Some(result), rest)),
      Err(_)              => Ok((None, data)),
    }
  }
}

pub struct RecursiveParser<I: ?Sized, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>>{
  parser: Rc<F>
}

impl<I:?Sized, O, F> SliceParser for RecursiveParser<I, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>> {

  type I = I;
  type O = O;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    (self.parser)().parse(data)
  }

}

impl<I:?Sized, O, F> ParserCombinator for RecursiveParser<I, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>> {}

impl<I: ?Sized, O, F> Clone for RecursiveParser<I, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>> {
  fn clone(&self) -> Self {
    RecursiveParser{parser: self.parser.clone()}
  }
}


pub struct MatchParser<T: Clone, U> {
  matcher: Rc<Box<Fn(T) -> Option<U>>>
}

impl<T: Clone, U> SliceParser for MatchParser<T,U> {
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

/// A Parser that will repeatedly parse `rep` and `sep` in sequence until `sep`
/// returns an error.  The accumulated `rep` results are returned.  If `rep`
/// returns an error at any time, the error is escelated.
pub struct RepSepParser<A,B> {
  pub rep: A,
  pub sep: B,
  pub min_reps: usize,
}
impl<I: ?Sized, A: SliceParser<I=I>, B: SliceParser<I=I>> SliceParser for RepSepParser<A,B> {
  type I = I;
  type O = Vec<A::O>;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    let mut remain = data;
    let mut v: Vec<A::O> = Vec::new();
    loop {
      match self.rep.parse(remain) {
        Ok((result, rest)) => {
          v.push(result);
          match self.sep.parse(rest.clone()) {
            Ok((_, rest2)) => {
              remain = rest2
            }
            Err(_) => {
              if v.len() < self.min_reps {
                return Err(format!("Not enough reps: required {}, got {}", self.min_reps, v.len()))
              } else {
                return Ok((v, rest))
              }
            }
          }
        }
        Err(err) => {
          return Err(format!("Error on rep: {}", err));
        }
      }
    }
  }
}

impl<I: ?Sized, A: ParserCombinator<I=I>, B: ParserCombinator<I=I>> ParserCombinator for RepSepParser<A,B> {}

impl<I: ?Sized, A: ParserCombinator<I=I>, B: ParserCombinator<I=I>> Clone for RepSepParser<A,B> {
  
  fn clone(&self) -> Self {
    RepSepParser{rep : self.rep.clone(), sep: self.sep.clone(), min_reps: self.min_reps}
  }

}



