//! Bitwise flags

#[macro_export]
macro_rules! flags_impl {
  ($name:ident,$unsigned:ty,$($field:tt,$value:expr),+) => {
    bitflags::bitflags! {
      #[derive(Copy, Clone, PartialEq, Eq)]
      struct $name: $unsigned {
        $(
          const $field = $value;
        )+
      }
    }

    impl std::fmt::Debug for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({:b})", self.bits()))?;
        bitflags::parser::to_writer(self, f)
      }
    }
  };
}

#[macro_export]
macro_rules! flags_builder_impl {
  ($builder:ident,$field:ident,$default:ident,$($lower:tt,$upper:path),+) => {
    impl $builder {
      $(
        pub fn $lower(&mut self, value: bool) -> &mut Self {
          let mut flags = self.$field.unwrap_or($default);
          if value {
            flags.insert($upper);
          } else {
            flags.remove($upper);
          }
          self.$field = Some(flags);
          self
        }
      )+
    }
  }
}
