// these imports (of, eg, 'literal') are only used within macros. It appears
// Rust will incorrectly warn that they are unused, hence this attribute.
#[allow(unused_imports)]
use parsers::*;

#[derive(Show, Eq, PartialEq, Clone)]
enum Input {
  A, 
  B, 
  C, 
  D,
}

//need more tests :/

#[test]
fn test_seq() {
  let input = [Input::A, Input::B, Input::C, Input::D];
  let parser = seq!(literal(Input::A), literal(Input::B), literal(Input::C));
  assert_eq!( parser.parse(input.as_slice()) , Ok(((Input::A, (Input::B, Input::C)), input.slice_from(3))));
}

#[test]
fn test_seq_map() {
  let input = [Input::A, Input::B];
  let parser = seq!(literal(Input::A), literal(Input::B) to |&: _| 5);
  let expected = Ok( (5, [].as_slice()) );
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_rep() {
  let input = [Input::A, Input::B, Input::A, Input::B, Input::A, Input::C];
  let parser = rep!(seq!(literal(Input::A), literal(Input::B)));
  let expected = Ok((
    vec![(Input::A, Input::B), (Input::A, Input::B)],
    input.slice_from(4)
  ));
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_or() {
  let input = [Input::A, Input::B, Input::A, Input::C];
  let parser = rep!(or!(literal(Input::A), literal(Input::B)));
  let expected = Ok((
    vec![Input::A, Input::B, Input::A],
    input.slice_from(3)
  ));
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_multi_or() {
  let input = [Input::A];
  let parser = or!(literal(Input::A), literal(Input::B), literal(Input::C));
  let expected = Ok( (Input::A, [].as_slice()) );
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_or_map() {
  let input = [Input::B];
  let parser = or!(literal(Input::A), literal(Input::B) to |&: _| 5);
  let expected = Ok( (5, [].as_slice()) );
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_map() {
  let input = [Input::A];
  let parser = map!(literal(Input::A), |&: _| 5);
  let expected = Ok( (5, [].as_slice()) );
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_recursive_or() {
  let input = [Input::A, Input::A, Input::C];
  fn a_seq<'a>() -> Box<Parser<'a, &'a [Input], usize> + 'a> {
    Box::new(or!(
      map!(literal(Input::C), |&: _| 2),
      map!(seq!(literal(Input::A), lazy!(a_seq())), |&: (_, seq)| 1 + seq)
    ))
  }
  let parser = a_seq();
  let expected = Ok( (4, [].as_slice()) );
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_repsep() {
  let input = [Input::A, Input::B, Input::C, Input::B, Input::A, Input::A];
  let parser = repsep!(or!(literal(Input::A), literal(Input::C)) , literal(Input::B));
  let expected = Ok((vec![Input::A, Input::C, Input::A], input.slice_from(5)));
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_opt() {
  let input = [Input::A, Input::A, Input::B];
  let parser = rep!(seq!(literal(Input::A), opt!(literal(Input::B))));
  let expected = Ok((vec![(Input::A, None), (Input::A, Some(Input::B))], input.slice_from(3)));
  assert_eq!( parser.parse(input.as_slice()), expected );
}

#[test]
fn test_matcher() {
  let input = [Input::A, Input::B, Input::C];
  let parser = rep!(matcher!(Input : Input::A => 4, Input::B => 5));
  let expected = Ok( (vec![4, 5], input.slice_from(2)) );
  assert_eq!( parser.parse(input.as_slice()), expected );

}
  
    
