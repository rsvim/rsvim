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

  (@from_int $name:tt,$ty:tt) => {
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

  (@from_str $name:tt,$ty:tt) => {
    impl From<String> for $name {
      fn from(value: String) -> Self {
        use compact_str::ToCompactString;
        Self(value.to_compact_string())
      }
    }

    impl From<compact_str::CompactString> for $name {
      fn from(value: CompactString) -> Self {
        Self(value)
      }
    }

    impl From<$name> for String {
      fn from(value: $name) -> Self {
        value.0.to_string()
      }
    }

    impl From<$name> for compact_str::CompactString {
      fn from(value: $name) -> Self {
        value.0
      }
    }

    impl From<&str> for $name {
      fn from(value: &str) -> Self {
        use compact_str::ToCompactString;
        Self(value.to_compact_string())
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

  (@incremental $name:tt,$atomic_int:tt,$plain_int:tt,$initial_value:expr) => {
    impl $name {
      pub fn next() -> Self {
        use std::sync::atomic::$atomic_int;

        static VALUE: $atomic_int = $atomic_int::new($initial_value);
        let v = VALUE
          .fetch_update(
            std::sync::atomic::Ordering::Relaxed,
            std::sync::atomic::Ordering::Relaxed,
            |x| {
              Some(if x == $plain_int::MAX {
                $initial_value
              } else {
                x + 1
              })
            },
          )
          .unwrap();
        Self::from(v)
      }
    }
  };

  (@unsigned $ty:tt,$name:tt,$initial:expr) => {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $name($ty);

    structural_id_impl!(@ord $name, $ty);
    structural_id_impl!(@eq $name, $ty);
    structural_id_impl!(@display $name, $ty);
    structural_id_impl!(@from_int $name, $ty);
    structural_id_impl!(@zero $name, $ty);

    paste::paste! {
      structural_id_impl!(@incremental $name, [< Atomic $ty:camel>], $ty, $initial);
    }
  };

  (@signed $ty:tt,$name:tt,$initial:expr) => {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $name($ty);

    structural_id_impl!(@ord $name, $ty);
    structural_id_impl!(@eq $name, $ty);
    structural_id_impl!(@display $name, $ty);
    structural_id_impl!(@from_int $name, $ty);
    structural_id_impl!(@zero $name, $ty);
    structural_id_impl!(@negative_one $name, $ty);

    paste::paste! {
      structural_id_impl!(@incremental $name, [< Atomic $ty:camel>], $ty, $initial);
    }
  };

  (usize,$name:tt,$initial:expr) => {
    structural_id_impl!(@unsigned usize,$name,$initial);
  };

  (u8,$name:tt,$initial:expr) => {
    structural_id_impl!(@unsigned u8,$name,$initial);
  };

  (i32,$name:tt,$initial:expr) => {
    structural_id_impl!(@signed usize,$name,$initial);
  };

  (i8,$name:tt,$initial:expr) => {
    structural_id_impl!(@signed i8,$name,$initial);
  };

  (str,$name:tt) => {
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct $name(CompactString);

    structural_id_impl!(@eq $name, CompactString);
    structural_id_impl!(@display $name, CompactString);
    structural_id_impl!(@from_str $name, CompactString);
  };
}
