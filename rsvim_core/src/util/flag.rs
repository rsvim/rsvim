//! Bitwise flags

#[macro_export]
macro_rules! flags_impl {
  ($name:ident,$unsigned:ty,$($field:tt),+) => {
    flags_impl!{@each($name,$unsigned,1){} $($field)+}
  };

  (@each($name:ident,$unsigned:ty,$($inc:tt)*){$($collect:tt)*} $i:ident $($rest:tt)*) => {
    flags_impl! {@each($name,$unsigned,$($inc)*<<1){
      $($collect)*
      const $i = $($inc)*;
    } $($rest)*}
  };

  (@each($name:ident,$unsigned:ty,$($inc:tt)*){$($collect:tt)*}) => {
    bitflags::bitflags! {
      struct $name: $unsigned {
        $($collect)*
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
  ($builder:ident,$field:ident,$default:expr,$($lower:tt,$upper:path),+) => {
    impl $builder {
      $(
        pub fn $lower(&mut self, value: bool) -> &mut Self {
          let mut flags = self.$field.unwrap_or($default);
          flags.set($upper, value);
          self.$field = Some(flags);
          self
        }
      )+
    }
  }
}
