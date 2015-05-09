use std::rc::Rc;

pub trait SliceParser  {
  type I;
  type O;

  fn parse<'a>(&self, data: &'a[Self::I]) -> ParseResult<&'a [Self::I], Self::O>;
}

pub trait ParserCombinator : SliceParser + Clone {

  fn then<P: SliceParser<I=Self::I>>(&self, p: P) -> ChainedParser<Self::I, Self,P> {
    ChainedParser{first: self.clone(), second: p}
  }

  fn then_r<P: ParserCombinator<I=Self::I>>(&self, p: P) -> MapParser<ChainedParser<Self::I, Self, P>, P::O> {
    self.then(p).map(|(_, t)| t)
  }

  fn then_l<P: ParserCombinator<I=Self::I>>(&self, p: P) -> MapParser<ChainedParser<Self::I, Self, P>, Self::O> {
    self.then(p).map(|(t, _)| t)
  }

  fn repeat(&self) -> RepeatParser<Self> {
    RepeatParser{parser: self.clone()}
  }
  
  fn map<T, F: 'static + Fn(Self::O) -> T>(&self, f: F) -> MapParser<Self, T> {
    MapParser{parser: self.clone(), mapper: Rc::new(Box::new(f))}
  }

  fn or<P: SliceParser<I=Self::I, O=Self::O>>(&self, p: P) -> OrParser<Self::I, Self::O, Self,P> {
    OrParser{first: self.clone(), second: p}
  }

}

pub type ParseResult<I,O> = Result<(O, I), String>;

pub struct ChainedParser<C, A: SliceParser<I=C>, B: SliceParser<I=C>> {
  first: A,
  second: B,
}
impl<C, A: SliceParser<I=C>, B: SliceParser<I=C>> SliceParser for ChainedParser<C, A, B> {
  type I = C;
  type O = (A::O,B::O);

  fn parse<'a>(&self, data: &'a[Self::I]) -> ParseResult<&'a [Self::I], Self::O>{
    match self.first.parse(data) {
      Ok((a, d2)) => match self.second.parse(d2) {
        Ok((b, remain)) => Ok(((a, b), remain)),
        Err(err) => Err(err)
      },
      Err(err) => Err(err)
    }
  }
}

impl<C, A: ParserCombinator<I=C>, B: ParserCombinator<I=C>>  Clone for ChainedParser<C, A, B> {
  
  fn clone(&self) -> Self {
    ChainedParser{first: self.first.clone(), second: self.second.clone()}
  }
}

impl<C, A: ParserCombinator<I=C>, B: ParserCombinator<I=C>>  ParserCombinator for ChainedParser<C, A, B> {}


pub fn lit<T: Eq + Clone>(l: T) -> LiteralParser<T> {
  LiteralParser{literal: l}
}
pub fn opt<T: SliceParser>(t: T) -> OptionParser<T> {
  OptionParser{parser: t}
}

#[derive(Clone)]
pub struct LiteralParser< T: Eq + Clone> {
  pub literal: T,
}

impl<T: Eq + Clone> SliceParser for LiteralParser< T> {
  type I = T;
  type O = T;

  fn parse<'a>(&self, data: &'a [T]) -> ParseResult<&'a [T], T> {
    if data.len() < 1 {
      return Err(format!("ran out of data"))
    }
    if data[0] == self.literal {
      Ok((data[0].clone(), data.tail()))
    } else {
      Err(format!("Literal mismatch"))
    }
  }
}

impl<T: Eq + Clone> ParserCombinator for LiteralParser<T>{}

#[derive(Clone)]
pub struct RepeatParser<P: SliceParser> {
  parser: P
}
impl<T: SliceParser> SliceParser for RepeatParser<T> {
  type I = T::I;
  type O = Vec<T::O>;
  
  fn parse<'a>(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
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

pub struct MapParser<P: SliceParser, T> {
  parser: P,
  mapper: Rc<Box<Fn(P::O) -> T>>,
}

impl<P: SliceParser, T> SliceParser for MapParser<P,T> {
  type I = P::I;
  type O = T;

  fn parse<'a>(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
    self.parser.parse(data).map(|(output, input)| ((self.mapper)(output), input))
  }

}

impl<P: ParserCombinator, T> Clone for MapParser<P,T> {

  fn clone(&self) -> Self {
    MapParser{parser: self.parser.clone(), mapper: self.mapper.clone()}
  }
}

impl<P: ParserCombinator, T> ParserCombinator for MapParser<P,T> {}

pub struct OrParser<I,O, S: SliceParser<I=I,O=O>, T: SliceParser<I=I,O=O>> {
  first: S,
  second: T,
}

impl<I,O, S: SliceParser<I=I,O=O>, T: SliceParser<I=I,O=O>> SliceParser for OrParser<I,O,S,T> {
  type I = I;
  type O = O;

  fn parse<'a>(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
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
pub struct OptionParser<P: SliceParser> {
  parser: P 
}
impl<P: SliceParser> SliceParser for OptionParser<P> {
  type I = P::I;
  type O = Option<P::O>;

  fn parse<'a>(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
    match self.parser.parse(data.clone()) {
      Ok((result, rest))  => Ok((Some(result), rest)),
      Err(_)              => Ok((None, data)),
    }
  }
}

pub struct RecursiveParser<I, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>>{
  parser: Rc<F>
}

impl<I, O, F> SliceParser for RecursiveParser<I, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>> {

  type I = I;
  type O = O;

  fn parse<'a>(&self, data: &'a [Self::I]) -> ParseResult<&'a [Self::I], Self::O> {
    (self.parser)().parse(data)
  }

}

impl<I, O, F> ParserCombinator for RecursiveParser<I, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>> {}

impl<I, O, F> Clone for RecursiveParser<I, O, F> where F: Fn() -> Box<SliceParser<I=I,O=O>> {
  fn clone(&self) -> Self {
    RecursiveParser{parser: self.parser.clone()}
  }
}


pub fn recursive<I,O, F:  Fn() -> Box<SliceParser<I=I,O=O>>>(f: F) -> RecursiveParser<I,O,F> {
  RecursiveParser{parser: Rc::new(f)}
}
