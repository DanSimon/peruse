use parsers::*;

#[deriving(Show)]
#[deriving(Eq)]
#[deriving(PartialEq)]
#[deriving(Clone)]
enum Input {
  A, B, C, D
}

//need more tests :/

#[test]
fn test_seq() {
  let input = [A, B, C, D];
  let parser = seq!(literal(A), literal(B), literal(C));
  assert_eq!( parser.parse(&input) , Ok(((A, (B, C)), input.slice_from(3))));
}

#[test]
fn test_rep() {
  let input = [A, B, A, B, A, C];
  let parser = rep!(seq!(literal(A), literal(B)));
  let expected = Ok((
    vec![(A, B), (A, B)],
    input.slice_from(4)
  ));
  assert_eq!( parser.parse(&input), expected );
}

#[test]
fn test_or() {
  let input = [A, B, A, C];
  let parser = rep!(or!(literal(A), literal(B)));
  let expected = Ok((
    vec![A, B, A],
    input.slice_from(3)
  ));
  assert_eq!( parser.parse(&input), expected );
}

#[test]
fn test_multi_or() {
  let input = [A];
  let parser = or!(literal(A), literal(B), literal(C));
  let expected = Ok( (A, [].as_slice()) );
  assert_eq!( parser.parse(&input), expected );
}

#[test]
fn test_map() {
  let input = [A];
  let parser = map!(literal(A), |&: a| 5u);
  let expected = Ok( (5u, [].as_slice()) );
  assert_eq!( parser.parse(&input), expected );
}

#[test]
fn test_recursive_or() {
  let input = [A, A, C];
  fn a_seq<'a>() -> Box<Parser<'a, &'a [Input], uint> + 'a> {
    box or!(
      map!(literal(C), |&: _| 2u),
      map!(seq!(literal(A), lazy!(a_seq())), |&: (a, seq)| 1u + seq)
    )
  }
  let parser = a_seq();
  let expected = Ok( (4u, [].as_slice()) );
  assert_eq!( parser.parse(&input), expected );

}

#[test]
fn test_repsep() {
  let input = [A, B, C, B, A, A];
  let parser = repsep!(or!(literal(A), literal(C)) , literal(B));
  let expected = Ok((vec![A, C, A], input.slice_from(5)));
  assert_eq!( parser.parse(&input), expected );
}

#[test]
fn test_opt() {
  let input = [A, A, B];
  let parser = rep!(seq!(literal(A), opt!(literal(B))));
  let expected = Ok((vec![(A, None), (A, Some(B))], input.slice_from(3)));
  assert_eq!( parser.parse(&input), expected );
}
    
