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

/// Usage Example:
///
/// ```rust
/// flags_builder_impl!(WinOptions, flags, line_break, wrap);
/// ```
///
/// Would generate:
///
/// ```rust
/// impl WinOptionsBuilder { // <- "WinOptions"Builder
///   // line_break
///   pub fn line_break(&mut self, value: bool) -> &mut Self { // <- "line_break"
///     let mut flags = self.flags.unwrap_or( FLAGS ); // <- self."flags", "FLAGS"
///     flags.set( Flags::LINE_BREAK , value); // <- "Flags"::"LINE_BREAK"
///     self.flags = Some(flags); // <- self."flags"
///     self
///   }
///   // wrap
///   pub fn wrap(&mut self, value: bool) -> &mut Self { // <- "wrap"
///     let mut flags = self.flags.unwrap_or( FLAGS ); // <- self."flags", "FLAGS"
///     flags.set( Flags::WRAP, value); // <- "Flags"::"WRAP"
///     self.flags = Some(flags); // <- self."flags"
///     self
///   }
/// }
/// ```
#[macro_export]
macro_rules! flags_builder_impl {
  ($builder:ident,$member:ident,$($field:ident),+) => {
    paste::paste! {
      impl [< $builder Builder >] {
        $(
          pub fn $field(&mut self, value: bool) -> &mut Self {
            let mut flags = self.$member.unwrap_or( [< $member:upper >] );
            flags.set( [< $member:camel >]::[<  $field:upper >] , value);
            self.$member = Some(flags);
            self
          }
        )+
      }
    }
  }
}
