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
      #[derive(Copy, Clone, PartialEq, Eq)]
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
  ($builder:ident,$member:ident,$flags:ident,$($field:ident),+) => {
    paste::paste! {
      impl [< $builder Builder >] {
        $(
          pub fn $field(&mut self, value: bool) -> &mut Self {
            let mut flags = self.$member.unwrap_or( [< $flags:snake:upper >] );
            flags.set( $flags::[<  $field:upper >] , value);
            self.$member = Some(flags);
            self
          }
        )+
      }
    }
  }
}
