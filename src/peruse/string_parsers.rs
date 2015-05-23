

use slice_parsers::*;
use regex::{Captures, Regex};
use std::rc::Rc;
    
/// A string Parser that attempts to consume the given regex
#[derive(Clone)]
pub struct RegexLiteralParser<T: Clone> {
  pub regex: Regex,
  literal: T,
}

impl<T: Clone> SliceParser for RegexLiteralParser<T> {
  type I = str;
  type O = T;

  fn parse<'a>(&self, data: &'a str) -> ParseResult<&'a str, Self::O>{
    self.regex.find(data).map(|(_, e)| (self.literal.clone(), &data[e..])).ok_or(format!("regex literal match fail"))
  }
}

impl<T: Clone> ParserCombinator for RegexLiteralParser<T> {}


pub struct RegexCapturesParser<T, F: Fn(Captures) -> T> {
  pub regex: Regex,
  f: Rc<Box<F>>
}

impl<T, F: Fn(Captures) -> T> SliceParser for RegexCapturesParser<T, F> {

  type I = str;
  type O = T;

  fn parse<'a>(&self, data: &'a str) -> ParseResult<&'a str, T> {
    match self.regex.captures(data) {
      Some(caps) => match caps.pos(0) {
        Some((_, e)) => Ok(((self.f)(caps), &data[e..])),
        None => Err(format!("No Match"))
      },
      None => Err(format!("No Match"))
    }
  }
}

impl<T: Clone, F: Fn(Captures) -> T> ParserCombinator for RegexCapturesParser<T, F> {}

impl<T: Clone, F: Fn(Captures) -> T> Clone for RegexCapturesParser<T, F> {

  fn clone(&self) -> Self {
    RegexCapturesParser{regex: self.regex.clone(), f: self.f.clone()}
  }
}


pub fn rlit<T: Clone>(r: Regex, l: T) -> RegexLiteralParser<T> {
  RegexLiteralParser{regex: r, literal: l}
}
pub fn str_lit<T: Clone>(s: &str, l: T) -> RegexLiteralParser<T> {
  let r = format!("^{}", s);
  let regex = Regex::new(&r).unwrap();
  RegexLiteralParser{regex: regex, literal: l}
}

pub fn capture<T, F: 'static + Fn(Captures) -> T>(reg: &str, f: F) -> RegexCapturesParser<T, F> {
  let regex = Regex::new(reg).unwrap();

  RegexCapturesParser{regex: regex, f: Rc::new(Box::new(f))}
}

