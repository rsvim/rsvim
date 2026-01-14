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
