
use std::ops::Fn;
use regex::{Captures, Regex};

/*
 * A Parser is designed to take an input type and turn part of it into an
 * output type.  Probably in every real-life scenario, the input type is a
 * slice of some type.  Thus a parser returns an output type along with another
 * input type, which for slices would be the rest of the data.
 */
pub trait Parser<'a, I, O> {

  fn parse(&self, data: I) -> ParseResult<'a, I, O>;

}

pub type ParseResult<'a, I:'a, O> = Result<(O, I), String>;

//coercion fn needed for unboxed closure macros
//without this all kinds of weird errors show up
pub fn coerce<'a, I, O>(l: Box<Parser<'a, I, O> + 'a>) -> Box<Parser<'a, I, O> + 'a> {
  l
}

pub fn literal<'a, T:'a + Eq + Clone>(literal: T) -> LiteralParser<'a, T>{
  LiteralParser{literal: literal}
}




/*
 * a parser that consumes one item in a slice and only returns ok if it equals
 * the given literal
 */
pub struct LiteralParser<'a, T:'a + Eq + Clone> {
  pub literal: T,
}

impl<'a, T: 'a + Eq + Clone> Parser<'a,  &'a [T], T> for LiteralParser<'a, T> {
  fn parse(&self, data: &'a[T]) -> ParseResult<'a, &'a[T], T> {
    if data.len() < 1 {
      return Err(format!("ran out of data"))
    }
    if data[0] == self.literal {
      Ok((data[0].clone(), data.slice_from(1)))
    } else {
      Err(format!("Literal mismatch"))
    }
  }
}

/*
 * A string Parser that attempts to consume the given regex
 */
pub struct RegexLiteralParser<'a> {
  pub regex: Regex,
}

impl<'a> Parser<'a, &'a str, ()> for RegexLiteralParser<'a> {
  fn parse(&self, data: &'a str) -> ParseResult<'a, &'a str, ()> {
    self.regex.find(data).map(|(_, e)| ((), data.slice_from(e))).ok_or(format!("regex literal match fail"))
  }
}

pub struct RegexCapturesParser<'a> {
  pub regex: Regex,
}
impl<'a> Parser<'a, &'a str, Captures<'a>> for RegexCapturesParser<'a> {
  fn parse(&self, data: &'a str) -> ParseResult<'a, &'a str, Captures<'a>> {
    match self.regex.captures(data) {
      Some(caps) => match caps.pos(0) {
        Some((_, e)) => Ok((caps, data.slice_from(e))),
        None => Err(format!("No Match"))
      },
      None => Err(format!("No Match"))
    }
  }
}





/*
 * A slice Parser that matches against the first item in the slice
 */
pub struct MatchParser<'a, I, O> {
  pub matcher: Box< Fn<(&'a I,), Result<O, String>> +'a>
}
impl<'a, I: Clone, O> Parser<'a, &'a [I], O> for MatchParser<'a, I, O> {
  fn parse(&self, data: &'a[I]) -> ParseResult<'a, &'a[I], O> {
    if data.len() < 1 {
      Err(format!("Unexpected End!"))
    } else {
      self.matcher.call((&data[0],)).map(|res| (res, data.slice_from(1)))
    }
  }
}

    

/*
 * A Parser that will keep repeating the given parser until it returns an
 * error.  The accumulated results are returned.
 */
pub struct RepParser<'a, I, O>{
  pub parser: &'a Parser<'a, I, O> + 'a
}

impl<'a, I: Clone, O> Parser<'a, I, Vec<O>> for RepParser<'a, I, O> {
  fn parse(&self, data: I) -> ParseResult<'a, I, Vec<O>> {
    let mut remain = data;
    let mut v: Vec<O> = Vec::new();
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

/*
 * A Parser that will repeatedly parse `rep` and `sep` in sequence until `sep`
 * returns an error.  The accumulated `rep` results are returned.  If `rep`
 * returns an error at any time, the error is escelated.
 */
pub struct RepSepParser<'a, I, O, U> {
  pub rep: &'a Parser<'a, I, O> + 'a,
  pub sep: &'a Parser<'a, I, U> + 'a,
  pub min_reps: uint,
}
impl<'a, I: Clone, O, U> Parser<'a, I, Vec<O>> for RepSepParser<'a, I, O, U> {
  fn parse(&self, data: I) -> ParseResult<'a, I, Vec<O>> {
    let mut remain = data;
    let mut v: Vec<O> = Vec::new();    
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
    unreachable!()
  }
}
  


/*
 * A Parser that will combine two parsers into a tuple of results.  If either
 * parser returns an error, the error is escelated (ie partial successes are
 * not returned)
 */
pub struct DualParser<'a, I, A, B> {
  pub first: &'a Parser<'a, I, A> + 'a,
  pub second: &'a Parser<'a, I, B> + 'a,
}

impl <'a, I, A, B> Parser<'a, I, (A,B)> for DualParser<'a, I, A, B> {
  
  fn parse(&self, data: I) -> ParseResult<'a, I, (A, B)> {
    match self.first.parse(data) {
      Ok((a, d2)) => match self.second.parse(d2) {
        Ok((b, remain)) => Ok(((a, b), remain)),
        Err(err) => Err(err)
      },
      Err(err) => Err(err)
    }
  }
}

/*
 * A parser that will attempt to parse using parser `a`, and then `b` if
 * the first fails.  The contained parsers are lazy so that we can support recursive
 * grammars.
 *
 * In general, the "greedier" parser should be `a`.
 */
pub struct OrParser<'a, I, O> {
  pub a: Box<Fn<(), Box<Parser<'a, I, O> + 'a> + 'a> + 'a>,
  pub b: Box<Fn<(), Box<Parser<'a, I, O> + 'a> + 'a> + 'a>
}

impl<'a, I: Clone, O> Parser<'a, I, O> for OrParser<'a, I, O> {
  fn parse(&self, data: I) -> ParseResult<'a, I, O> {
    match self.a.call(()).parse(data.clone()) {
      Ok((a, d2)) => Ok((a, d2)),
      Err(_) => match self.b.call(()).parse(data.clone()) {
        Ok((b, remain)) => Ok((b, remain)),
        Err(err) => Err(err)
      }
    }
  }
}

/*
 * A Parser that can map the successful result of a parser to another type
 */
pub struct MapParser<'a, I, O, U> {
  pub parser: &'a Parser<'a, I, O> + 'a,
  pub mapper: &'a Fn<(O,), U> + 'a, //this has to be a &Fn and not a regular lambda since it must be immutable
}
impl<'a, I, O, U> Parser<'a, I, U> for MapParser<'a, I, O, U> {
  fn parse(&self, data: I) -> ParseResult<'a, I, U> {
    self.parser.parse(data).map(|(output, input)| ((self.mapper.call((output,)), input)))
  }
}

/*
 * A Parser that will attempt to use a parser, returning an option of the contained parser's result
 */
pub struct OptionParser<'a, I, O> {
  pub parser: &'a Parser<'a, I, O> + 'a
}
impl<'a, I: Clone, O> Parser<'a, I, Option<O>> for OptionParser<'a, I, O> {
  fn parse(&self, data: I) -> ParseResult<'a, I, Option<O>> {
    match self.parser.parse(data.clone()) {
      Ok((result, rest))  => Ok((Some(result), rest)),
      Err(_)              => Ok((None, data)),
    }
  }
}

      
