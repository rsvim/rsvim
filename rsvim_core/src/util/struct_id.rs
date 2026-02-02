//! Structured ID.

#[macro_export]
macro_rules! structural_id_impl {
  (@eq $name:tt,$ty:tt) => {
    impl PartialEq<$ty> for $name {
      fn eq(&self, other: &$ty) -> bool {
        self.0.eq(other)
      }
    }
  };

  (@ord $name:tt,$ty:tt) => {
    impl PartialOrd<$ty> for $name {
      fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
      }
    }
  };

  (@display $name:tt,$ty:tt) => {
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
  };

  (@convert $name:tt,$ty:tt) => {
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

  (@zero $name:tt,$ty:tt) => {
    impl $name {
      pub const fn zero() -> Self {
        Self(0)
      }
    }
  };

  (@negative_one $name:tt,$ty:tt) => {
    impl $name {
      pub const fn negative_one() -> Self {
        Self(-1)
      }
    }
  };

  (unsigned,$name:tt,$ty:tt) => {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $name($ty);

    structural_id_impl!(@ord $name, $ty);
    structural_id_impl!(@eq $name, $ty);
    structural_id_impl!(@display $name, $ty);
    structural_id_impl!(@convert $name, $ty);
    structural_id_impl!(@zero $name, $ty);
  };

  (signed,$name:tt,$ty:tt) => {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $name($ty);

    structural_id_impl!(@ord $name, $ty);
    structural_id_impl!(@eq $name, $ty);
    structural_id_impl!(@display $name, $ty);
    structural_id_impl!(@convert $name, $ty);
    structural_id_impl!(@zero $name, $ty);
    structural_id_impl!(@negative_one $name, $ty);
  };

  (stringify,$name:tt,$ty:tt) => {
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct $name($ty);

    structural_id_impl!(@eq $name, $ty);
    structural_id_impl!(@display $name, $ty);
    structural_id_impl!(@convert $name, $ty);
  };
}

#[macro_export]
macro_rules! next_incremental_id_impl {
  ($func_name:ident,$struct_name:ident,$atomic_int:tt,$plain_int:tt,$initial:expr) => {
    pub fn $func_name() -> $struct_name {
      static VALUE: $atomic_int = $atomic_int::new($initial);
      let v = VALUE
        .fetch_update(
          std::sync::atomic::Ordering::Relaxed,
          std::sync::atomic::Ordering::Relaxed,
          |x| {
            Some(if x == $plain_int::MAX {
              $initial
            } else {
              x + 1
            })
          },
        )
        .unwrap();
      $struct_name::from(v)
    }
  };
}
