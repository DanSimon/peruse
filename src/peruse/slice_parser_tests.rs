use slice_parsers::*;

#[test]
fn test_literal() {
  let parser = lit(4);
  let input = [4, 3];
  assert_eq!(parser.parse(&input), Ok((4, &input[1..])));
}

#[test]
fn test_then() {
  let parser = lit(1).then(lit(2));
  let input = [1, 2, 3];
  assert_eq!(parser.parse(&input), Ok(((1, 2), &input[2..])));
}

#[test]
fn test_map() {
  let parser = lit(1).then(lit(2)).map(|(a, b)| a + b);
  let input = [1, 2, 3];
  assert_eq!(parser.parse(&input), Ok((3, &input[2..])));
}

#[test]
fn test_repeat() {
  let parser = lit(1).repeat();
  let input = [1, 1, 1, 2];
  assert_eq!(parser.parse(&input), Ok((vec![1, 1, 1], &input[3..])));
}

#[test]
fn test_or() {
  let parser = lit(1).or(lit(0)).repeat();
  let input = [1, 1, 0, 1, 2];
  assert_eq!(parser.parse(&input), Ok((vec![1, 1, 0, 1], &input[4..])));
}

#[test]
fn test_recursive() {
  fn recurse() -> Box<SliceParser<I=i32, O=i32>> {
    let end = lit(1).map(|_| 0);
    let rec = lit(0).then_r(recursive(|| recurse())).map(|t| t + 1);
    Box::new(end.or(rec))
  }
  let input = [0,0,0,1, 2];

  assert_eq!(recurse().parse(&input), Ok((3, &input[4..])));

}
