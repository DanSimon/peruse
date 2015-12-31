use utils::*;
use parsers::*;
use slice_parsers::*;

#[test]
fn test_skip() {
  let parser = skip(lit(4), one_of(vec![lit(1), lit(2)])).repeat();
  let input = [1, 4, 2, 1, 4, 4, 3];
  assert_eq!(parser.parse(&input), Ok((vec![4, 4, 4], &input[6..])));
}