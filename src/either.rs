pub enum Either<U, V> {
  Left(U),
  Right(V),
}

impl<U, V> Either<U, V> {
  pub fn map_left<F, R>(self, f: F) -> Either<R, V>
  where
    F: Fn(U) -> R,
  {
    return match self {
      Either::Left(l) => Either::Left(f(l)),
      Either::Right(r) => Either::Right(r),
    };
  }
  pub fn left_or(self, u: U) -> U {
    return match self {
      Either::Left(l) => l,
      _ => u,
    };
  }
  pub fn unwrap_left(self) -> U {
    match self {
      Either::Left(l) => l,
      _ => panic!(),
    }
  }

  pub fn map_right<F, R>(self, f: F) -> Either<U, R>
  where
    F: Fn(V) -> R,
  {
    return match self {
      Either::Left(l) => Either::Left(l),
      Either::Right(r) => Either::Right(f(r)),
    };
  }
  pub fn right_or(self, v: V) -> V {
    return match self {
      Either::Right(r) => r,
      _ => v,
    };
  }
  pub fn unwrap_right(self) -> V {
    match self {
      Either::Right(r) => r,
      _ => panic!(),
    }
  }
}

pub fn left<U, V>(u: U) -> Either<U, V> {
  Either::Left(u)
}

pub fn right<U, V>(v: V) -> Either<U, V> {
  Either::Right(v)
}
