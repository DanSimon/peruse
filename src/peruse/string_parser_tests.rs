use slice_parsers::*;
use string_parsers::*;
use std::str::FromStr;

#[test]
fn test_literal() {
  let parser = (str_lit("a", 3).or(str_lit("b", 4))).repeat();
  let data = "babac";
  assert_eq!(parser.parse(data), Ok((vec![4,3,4,3], "c")));
}

#[test]
fn test_captures() {
  let parser = capture(r"(\d+)", |caps| <i32>::from_str(caps.at(1).unwrap()).unwrap());
  let data = "34bah";
  assert_eq!(parser.parse(data), Ok((34, "bah")));
}
