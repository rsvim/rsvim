//! Structured ID.

#[macro_export]
macro_rules! struct_id_impl {
  ($name:ident,$ty:ty) => {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $name($ty);

    impl PartialOrd<$ty> for $name {
      fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
      }
    }

    impl PartialEq<$ty> for $name {
      fn eq(&self, other: &$ty) -> bool {
        self.0.eq(other)
      }
    }

    impl std::fmt::Debug for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
      }
    }

    impl From<$ty> for $name {
      fn from(value: $ty) -> Self {
        Self(value)
      }
    }

    impl From<$name> for $ty {
      fn from(value: $name) -> Self {
        value.0
      }
    }

    impl $name {
      pub const fn zero() -> Self {
        Self(0)
      }

      pub fn value(&self) -> $ty {
        self.0
      }
    }
  };

  ($name:ident,$ty:ty,negative) => {
    struct_id_impl!($name, $ty);

    impl $name {
      pub const fn negative_one() -> Self {
        Self(-1)
      }
    }
  };
}

#[macro_export]
macro_rules! next_incremental_id_impl {
  ($func_name:ident,$struct_name:ident,$atomic_int:tt,$plain_int:tt) => {
    pub fn $func_name() -> $struct_name {
      static VALUE: $atomic_int = $atomic_int::new(1);
      let v = VALUE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
          Some(if x == $plain_int::MAX { 1 } else { x + 1 })
        })
        .unwrap();
      $struct_name::from(v)
    }
  };
}
