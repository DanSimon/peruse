use parsers::*;
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
fn test_then_l() {
  let parser = lit(1).then_l(lit(2));
  let input = [1, 2, 3];
  assert_eq!(parser.parse(&input), Ok((1, &input[2..])));
}


#[test]
fn test_then_r() {
  let parser = lit(1).then_r(lit(2));
  let input = [1, 2, 3];
  assert_eq!(parser.parse(&input), Ok((2, &input[2..])));
}


#[test]
fn test_map() {
  let input = [1, 2, 3];
  let parser = lit(1).then(lit(2)).map(|(a, b)| a + b);
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
  fn recurse() -> Box<Parser<I=[i32], O=i32>> {
    let end = lit(1).map(|_| 0);
    let rec = lit(0).then_r(recursive(|| recurse())).map(|t| t + 1);
    Box::new(end.or(rec))
  }
  let input = [0,0,0,1, 2];

  assert_eq!(recurse().parse(&input), Ok((3, &input[4..])));

}

#[test]
fn test_opt() {
  let parser = opt(lit(1));
  let input1 = [0, 1];
  let input2 = [1, 0];

  assert_eq!(parser.parse(&input1), Ok((None, &input1[0..])));
  assert_eq!(parser.parse(&input2), Ok((Some(1), &input2[1..])));
}

#[test]
fn test_match() {
  let parser = matcher(|i| if i < 4 {Some(i)} else {None}).repeat();
  let input = [1, 2, 3, 4, 5];
  assert_eq!(parser.parse(&input), Ok((vec![1, 2, 3], &input[3..])));
}

#[test]
fn test_readme_code() {

//let's start with something simple, a parser that looks for one particular
//integer as the first element of a given slice

let p1 = lit(3);

//calling parse will return a ParseResult, containing the parsed value along
//with a slice of any unparsed data

println!("{:?}", p1.parse(&[3, 1, 2]) );
//Ok((3, [1, 2]))

println!("{:?}", p1.parse(&[4, 1, 2]) );
//Err("Literal mismatch")

//now we can start to chain parsers together

let p2 = lit(3).or(lit(4));

println!("{:?}", p2.parse(&[4, 1, 2]) );
//Ok((4, [1, 2]))

//and turn the parsed items into other types

let p3 = lit(3).or(lit(4)).then(lit(1)).map(|(a, b)| a + b);

println!("{:?}", p3.parse(&[4, 1, 2]) );
//Ok((5, [2]))


//let's say we have the following array
let arr = [1, 0, 1, 0, 1, 0];

//how about we write a parser to count the number of sequences of 1, 0

let p4 = lit(1).then(lit(0)).repeat().map(|v| v.len());

println!("{:?}", p4.parse(&arr)); 
//Ok((3, []))

//lastly we can define a recursive parser in a static function
fn count_zeros() -> Box<SliceParser<i32, i32>> {
  let end = lit(1).map(|_| 0);
  let rec = lit(0).then_r(recursive(count_zeros)).map(|t| t + 1);
  Box::new(end.or(rec))
}

println!("{:?}",count_zeros().parse(&[0,0,0,0,0,1]));
//Ok((5, []))



}

#[test]
fn basic_example() {
  #[derive(Debug, Clone, Eq, PartialEq)]
  enum Token {
    PlusSign,
    MultSign,
    OpenParen,
    CloseParen,
    Term(i32)
  }
  #[derive(Clone, Debug, Eq, PartialEq)]
  enum Expression {
    Plus(Vec<Expression>),
    Mult(Vec<Expression>),
    Term(i32),
  }

  fn eval(expr: &Expression) -> i32 {
    match  *expr {
      Expression::Plus(ref v) => {
        let mut sum = 0;
        for e in v.iter() {
          sum += eval(e);
        }
        sum
      },
      Expression::Mult(ref v) => {
        let mut prod = 1;
        for e in v.iter() {
          prod *= eval(e);
        }
        prod
      },
      Expression::Term(t) => t
    }
  }


  fn expression() -> Box<Parser<I=[Token], O=Expression>> {

    let paren = lit(Token::OpenParen).then_r(recursive(|| expression())).then_l(lit(Token::CloseParen));

    let term = || paren.or(matcher(|t| match t {
      Token::Term(t) => Some(Expression::Term(t)),
      _ => None
    }));

    let mult = repsep(term(), lit(Token::MultSign)).map(|v| if v.len() == 1 {v[0].clone()} else {Expression::Mult(v)});

    let add = repsep(mult, lit(Token::PlusSign)).map(|v| if v.len() == 1 {v[0].clone()} else {Expression::Plus(v)});

    Box::new(add)
  }

  let input = [Token::Term(4), Token::MultSign, Token::Term(5)];
  let expected = Expression::Mult(vec![Expression::Term(4), Expression::Term(5)]);
  let (res, _) = expression().parse(&input).unwrap();
  assert_eq!(res, expected);

  // (3 + 4) * (4 + 5 + 6)
  let input2 = [Token::OpenParen, Token::Term(3), Token::PlusSign, Token::Term(4), Token::CloseParen, Token::MultSign, Token::OpenParen, Token::Term(4), Token::PlusSign, Token::Term(5), Token::PlusSign, Token::Term(6), Token::CloseParen];

  let (res, _ ) = expression().parse(&input2).unwrap();
  assert_eq!(eval(&res), 105);
}

#[test]
fn test_oneof() {

  //this has to be a long list to make sure we don't hit an issue with rustc taking forever
  let p = one_of(vec![
    lit(1), 
    lit(3), 
    lit(5), 
    lit(7), 
    lit(9), 
    lit(11), 
    lit(13), 
    lit(15), 
    lit(17),
    lit(19),
    lit(21),
    lit(23),
    lit(25),
    lit(27),
    lit(29),
    lit(31),
    lit(33),
    lit(21),
    lit(23),
    lit(25),
    lit(27),
    lit(29),
    lit(31),
    lit(33),
    lit(21),
    lit(23),
    lit(25),
    lit(27),
    lit(29),
    lit(31),
    lit(33),
    lit(21),
    lit(23),
    lit(25),
    lit(27),
    lit(29),
    lit(31),
    lit(33),
  ]);

  let input = [3, 1, 9, 11, 27, 2];

  assert_eq!(p.repeat().parse(&input), Ok((vec![3, 1, 9, 11, 27], &input[5..])));

}

#[test]
fn test_keep_skip() {
  let parser = keep_skip(lit(4), one_of(vec![lit(1), lit(2)])).repeat();
  let input = [1, 4, 2, 1, 4, 4, 3];
  assert_eq!(parser.parse(&input), Ok((vec![4, 4, 4], &input[6..])));
}