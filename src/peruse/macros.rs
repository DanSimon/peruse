
use parsers::*;


#[macro_export]
macro_rules! or {
  ($a: expr, $b: expr) => {
    OrParser{
      a: $a,
      b: $b,
    } 
 };
  ($a: expr, $b: expr $(, $c: expr)* ) => {
    OrParser{
      a: $a,
      b: or!($b, $($c),*),
    } 
  };
}

#[macro_export]
pub macro_rules! seq {
  ($a: expr, $b: expr ) => {
    DualParser{
      first: $a,
      second: $b,
    }
 };
  ($a: expr, $b: expr $(, $c: expr)* ) => {
    DualParser{
      first: $a,
      second: seq!($b, $($c),* ),
    }
  };
}

#[macro_export]
pub macro_rules! map {
  ($a: expr, $b: expr) => {
    MapParser{
      parser: $a,
      mapper: box $b
    }
  }
}

#[macro_export]
pub macro_rules! repsep {
  ($rep: expr, $sep: expr, $min: expr) => {
    RepSepParser{
      rep: $rep,
      sep: $sep,
      min_reps: $min,
    }
  };
  ($rep: expr, $sep: expr) => {
    repsep!($rep, $sep, 0)
  };
}

#[macro_export]
pub macro_rules! rep {
  ($rep: expr) => {
    RepParser{
      parser: $rep,
    }
  }
}

#[macro_export]
pub macro_rules! opt {
  ($rep: expr) => {
    OptionParser{
      parser: $rep,
    }
  }
}

#[macro_export]
pub macro_rules! lazy {
  ($rep: expr) => {
    LazyParser{
      generator: box |&:| $rep
    }
  }
}
