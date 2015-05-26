use std::rc::Rc;


/////////     TRAITS/TYPES       //////////

/// The base trait for any parser.  
pub trait Parser  {
  type I: ?Sized;
  type O;

  /// Attempt to parse an input value into an output value
  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O>;
}

/// Combinator methods for slice parsers.  In most cases, these methods copy
/// the caller into a higher-order parser
pub trait ParserCombinator : Parser + Clone {

  /// Chain this parser with another parser, creating new parser that returns a
  /// tuple of their results
  fn then<P: Parser<I=Self::I>>(&self, p: P) -> ChainedParser<Self,P> {
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
  fn or<P: Parser<I=Self::I, O=Self::O>>(&self, p: P) -> OrParser<Self,P> {
    OrParser{first: self.clone(), second: p}
  }


}

/// The result of a parser's attempt to parse input data.  
///
/// A successful result contains the output value of the parser along with a new input value that
/// can be consumed by subsequent parsers.  A failed result contains an error message.
pub type ParseResult<I,O> = Result<(O, I), String>;

/////////     FUNCTIONS     ///////////

/// Create a parser that will return Some if the given parser is successful, None otherwise
///
/// # Examples
/// This parser will simply return a failure
/// ```no_run
/// # use peruse::parsers::*;
/// # use peruse::slice_parsers::lit;
/// let input = ["a", "b" , "c"];
/// let parser = lit("d");
/// parser.parse(&input); //Err
/// ```
/// But this will be return an `Ok((None, ...))`
/// ```no_run
/// # use peruse::parsers::*;
/// # use peruse::slice_parsers::lit;
/// let input = ["a", "b" , "c"];
/// let parser = opt(lit("d"));
/// parser.parse(&input); //Ok
/// ```
///
pub fn opt<T: Parser>(t: T) -> OptionParser<T> {
  OptionParser{parser: t}
}

/// Create a lazily evaluated parser from a function.  This can be used to generate recursive parsers
///
/// # Examples
///
/// ```
/// # use peruse::parsers::*;
/// # use peruse::slice_parsers::lit;
/// 
/// fn recurse() -> Box<Parser<I=[i32], O=i32>> {
///   let end = lit(1).map(|_| 0);
///   let rec = lit(0).then_r(recursive(|| recurse())).map(|t| t + 1);
///   Box::new(end.or(rec))
/// }
/// let input = [0,0,0,1, 2];
/// # assert_eq!(recurse().parse(&input), Ok((3, &input[4..])));
/// ```
///
pub fn recursive<I:?Sized,O, F:  Fn() -> Box<Parser<I=I,O=O>>>(f: F) -> RecursiveParser<I,O,F> {
  RecursiveParser{parser: Rc::new(f)}
}

/// Create a parser that will repeatedly use the `rep` and `sep` parsers in
/// sequence, building a vector of results from `rep`.  This will repeat until
/// `sep` returns an error.  If at any point `rep` returns an error, the collected
/// values are discarded and the error is escelated.
///
/// # Examples
/// ```no_run
/// # use peruse::parsers::*;
/// # use peruse::slice_parsers::lit;
/// let input = [0,1,0,1,0,4];
/// let parser = repsep(lit(0), lit(1));
/// let res = parser.parse(&input);
/// // Ok((Vec[0,0,0], [4]))
///
/// let bad_input = [0,1,0,1,2,1];
/// let res2 = parser.parse(&bad_input);
/// // Err
/// ```
pub fn repsep<I: ?Sized, A: Parser<I=I>, B: Parser<I=I>>(rep: A, sep: B) -> RepSepParser<A,B> {
  RepSepParser{rep: rep, sep: sep, min_reps: 1}
}

/// Create a parser that attempts to use each of the given parsers until one succeeds.  If all the
/// given parses are literally the exact same type, they can be unboxed, otherwise you'll have to
/// box them using the `boxed` function.
///
/// # Examples
/// 
/// Here both parsers have the same structure, and can be used unboxed
///
/// ```no_run
/// # use peruse::parsers::*;
/// # use peruse::slice_parsers::lit;
/// let input = [2,3,4];
/// let p1 = lit(2);
/// let p2 = lit(3);
/// let parser = one_of(vec![p1, p2]);
/// parser.parse(&input);
/// ```
/// 
/// These parsers have different structure, thus different types, so they need to be boxed to be
/// used.
///
/// ```no_run
/// # use peruse::parsers::*;
/// # use peruse::slice_parsers::lit;
/// let input = [2, 3, 4];
/// let p1 = lit(2).then(lit(3)).map(|(a, b)| a * b);
/// let p2 = lit(4);
/// let parser = one_of(vec![boxed(p1), boxed(p2)]);
/// parser.parse(&input);
/// ```
///
pub fn one_of<T: Parser>(t: Vec<T>) -> OneOfParser<T> {
  OneOfParser{options: t}
}

/// Wrap a boxed parser.  This mostly exists to avoid slow compile times.  Boxing a complex parser into a
/// trait object keep compile times down as the boxed parser is combined with other parsers
///
/// # Examples
/// ```no_run
/// # use peruse::parsers::*;
/// # use peruse::slice_parsers::lit;
/// let p1 = lit(1).or(lit(2)).or(lit(3)).or(lit(4)).or(lit(5)).or(lit(6));
/// let p2 = lit(7).or(lit(8)).or(lit(9)).or(lit(10)).or(lit(11)).or(lit(12));
/// let p3 = boxed(p1).or(boxed(p2));
/// ```
pub fn boxed<I: ?Sized,O, P:'static + Parser<I=I, O=O> >(p: P) -> BoxedParser<I,O> {
  BoxedParser{parser: Rc::new(Box::new(p))}
}


////////////    STRUCTS     //////////////


/// A Chained parser contains two parsers that will be used in sequence to
/// create a tuple of parsed values
pub struct ChainedParser<A,B> {
  first: A,
  second: B,
}
impl<C: ?Sized, A: Parser<I=C>, B: Parser<I=C>> Parser for ChainedParser<A, B> {
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


/// A Parser that repeats the given parser until it encounters an error.  A
/// vector of the accumulated parsed values is returned
pub struct RepeatParser<P: Parser> {
  parser: P
}
impl<T: Parser> Parser for RepeatParser<T> {
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
pub struct MapParser<I: ?Sized, P: Parser<I=I>, T> {
  parser: P,
  mapper: Rc<Box<Fn(P::O) -> T>>,
}

impl<I: ?Sized, P: Parser<I=I>, T> Parser for MapParser<I,P,T> {
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

pub struct OrParser<S: Parser,T: Parser> {
  first: S,
  second: T,
}

impl<I:?Sized,O, S: Parser<I=I,O=O>, T: Parser<I=I,O=O>> Parser for OrParser<S,T> {
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
pub struct OptionParser<P: Parser> {
  parser: P 
}
impl<P: Parser> Parser for OptionParser<P> {
  type I = P::I;
  type O = Option<P::O>;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    match self.parser.parse(data.clone()) {
      Ok((result, rest))  => Ok((Some(result), rest)),
      Err(_)              => Ok((None, data)),
    }
  }
}

impl<P: ParserCombinator> ParserCombinator for OptionParser<P> {}

pub struct RecursiveParser<I: ?Sized, O, F> where F: Fn() -> Box<Parser<I=I,O=O>>{
  parser: Rc<F>
}

impl<I:?Sized, O, F> Parser for RecursiveParser<I, O, F> where F: Fn() -> Box<Parser<I=I,O=O>> {

  type I = I;
  type O = O;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    (self.parser)().parse(data)
  }

}

impl<I:?Sized, O, F> ParserCombinator for RecursiveParser<I, O, F> where F: Fn() -> Box<Parser<I=I,O=O>> {}

impl<I: ?Sized, O, F> Clone for RecursiveParser<I, O, F> where F: Fn() -> Box<Parser<I=I,O=O>> {
  fn clone(&self) -> Self {
    RecursiveParser{parser: self.parser.clone()}
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
impl<I: ?Sized, A: Parser<I=I>, B: Parser<I=I>> Parser for RepSepParser<A,B> {
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


/// A Parser that takes a vector of parsers (of the exact same type) and
/// returns the value from the first parser to return a non-error.  This parser
/// solely exists because doing a or b or c or d... ends up crushing rustc
#[derive(Clone)]
pub struct OneOfParser<T: Parser> {
  options: Vec<T>
}

impl<T: Parser> Parser for OneOfParser<T> {
  type I = T::I;
  type O = T::O;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    for p in self.options.iter() {
      let r = p.parse(data.clone());
      if r.is_ok() {
        return r;
      }
    }
    Err(format!("All options failed"))
  }

}

impl<T: ParserCombinator> ParserCombinator for OneOfParser<T> {}


/// this parser solely exists to avoid insanely long compile times in rustc.
/// When you have a fairly large parser, it's best to box it.  Yes we're
/// introducing extra dynamic dispatch, but only on a small amount.  In some
/// cases this is the only way to get rustc to not take (literally) a million
/// years!
pub struct BoxedParser<I:?Sized,O> {
  parser: Rc<Box<Parser<I=I,O=O>>>
}

impl<I:?Sized, O> Parser for BoxedParser<I, O> {

  type I = I;
  type O = O;

  fn parse<'a>(&self, data: &'a Self::I) -> ParseResult<&'a Self::I, Self::O> {
    self.parser.parse(data)
  }

}

impl<I:?Sized, O> ParserCombinator for BoxedParser<I, O>  {}

impl<I: ?Sized, O> Clone for BoxedParser<I, O>  {
  fn clone(&self) -> Self {
    BoxedParser{parser: self.parser.clone()}
  }
}
