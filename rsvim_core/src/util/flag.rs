//! Bitwise flags

#[macro_export]
macro_rules! flags_impl {
  ($name:ident,$unsigned:ty,$($upperfield:tt,$value:literal,$lowerfield:tt),*) => {
    bitflags::bitflags! {
      #[derive(Copy, Clone, PartialEq, Eq)]
      struct $name: $unsigned {
        $(
          const $upperfield = $value;
        )*
      }
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
            self.insert($name::$upperfield);
          } else {
            self.remove($name::$upperfield);
          }
        }
      )*
      }
    }
  };
}

#[macro_export]
macro_rules! flags_builder_impl {
  ($name:ident,$flags:ident,$default_flags:ident,$($lowerfield:tt,$upperfield:tt),*) => {
    $(
      pub fn $lowerfield(&mut self, value: bool) -> &mut Self {
        let mut flags = self.$flags.unwrap_or($default_flags);
        if value {
          flags.insert($name::$upperfield);
        } else {
          flags.remove($name::$upperfield);
        }
        self.$flags = Some(flags);
        self
      }
    )*
  }
}
