//! Bitwise flags

#[macro_export]
macro_rules! flags_impl {
  ($name:ident,$unsigned:ty,$($field:tt,$value:literal),*) => {
    #[derive(Copy, Clone, PartialEq, Eq)]
    struct $name: $unsigned {
      $(
        const $field = $value;
      )*
    }
  };
}
