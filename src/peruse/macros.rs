

#[macro_export]
pub macro_rules! map {
  ($a: expr, $b: expr) => {
    ::peruse::parsers::MapParser{
      parser: $a,
      mapper: box $b
    }
  }
}

#[macro_export]
macro_rules! or {
  ($a: expr, $b: expr) => {
    ::peruse::parsers::OrParser{
      a: $a,
      b: $b,
    } 
 };
  ($a: expr, $b: expr, $($c: expr),* ) => {
    ::peruse::parsers::OrParser{
      a: $a,
      b: or!($b, $($c),*),
    } 
  };
  ($a: expr, $($b: expr),+ to $mapper: expr) => {
    map!(or!($a, $($b),+), $mapper)
  }

}

#[macro_export]
pub macro_rules! seq {
  ($a: expr, $b: expr ) => {
    ::peruse::parsers::DualParser{
      first: $a,
      second: $b,
    }
 };
  ($a: expr, $b: expr, $($c: expr),* ) => {
    ::peruse::parsers::DualParser{
      first: $a,
      second: seq!($b, $($c),* ),
    }
  };
  ($a: expr, $($b: expr),+ to $mapper: expr) => {
    map!(seq!($a, $($b),+), $mapper)
  }
}


#[macro_export]
pub macro_rules! repsep {
  ($rep: expr, $sep: expr, $min: expr) => {
    ::peruse::parsers::RepSepParser{
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
    ::peruse::parsers::RepParser{
      parser: $rep,
    }
  }
}

#[macro_export]
pub macro_rules! opt {
  ($rep: expr) => {
    ::peruse::parsers::OptionParser{
      parser: $rep,
    }
  }
}

#[macro_export]
pub macro_rules! lazy {
  ($rep: expr) => {
    ::peruse::parsers::LazyParser{
      generator: box |&:| $rep
    }
  }
}

#[macro_export]
pub macro_rules! matcher {
  ($t: ty : $($m: pat => $map: expr),+) => {
    ::peruse::parsers::MatchParser{
      matcher: box |&: i: &$t| match *i {
        $(
        $m => Ok($map),
        )+
        other => Err(format!("Match error"))
      }
    }
  }
}
