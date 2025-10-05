//! Bitwise flags

#[macro_export]
macro_rules! flags_impl {
  ($name:ident,$unsigned:ty,$($upperfield:tt,$value:literal,$lowerfield:tt),*) => {
    #[derive(Copy, Clone, PartialEq, Eq)]
    struct $name: $unsigned {
      $(
        const $upperfield = $value;
      )*
    }

    impl std::fmt::Debug for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
      }
    }

    paste::paste! {
    impl $name {
    $(
      pub fn $lowerfield(&self) -> bool {
        self.contains($name::$upperfield)
      }

      pub fn [<set_ $lowerfield>](&mut self, value: bool) {
        if value {
          self.insert(&name::&upperfield);
        } else {
          self.remove(&name::&upperfield);
        }
      }
    )*
    }
    }
  };
}
