//! TextDecoder and its options

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_impl;
use crate::js::converter::*;
use crate::to_v8_prop;

flags_impl!(Flags, u8, FATAL, IGNORE_BOM);

/// Option names.
pub const FATAL: &str = "fatal";
pub const IGNORE_BOM: &str = "ignoreBOM";
pub const ENCODING: &str = "encoding";

/// Default option values.
pub const FATAL_DEFAULT: bool = false;
pub const IGNORE_BOM_DEFAULT: bool = false;
pub const ENCODING_DEFAULT: &str = "utf-8";

// fatal=false
// ignoreBOM=false
const FLAGS: Flags = Flags::empty();

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct TextDecoder {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // fatal
  // ignoreBOM
  flags: Flags,

  pub encoding: String,
}

flags_builder_impl!(TextDecoder, flags, fatal, ignore_bom);

impl TextDecoder {
  pub fn fatal(&self) -> bool {
    self.flags.contains(Flags::FATAL)
  }

  pub fn ignore_bom(&self) -> bool {
    self.flags.contains(Flags::IGNORE_BOM)
  }
}

from_v8_impl!(
  TextDecoder,
  [(String, encoding), (bool, fatal), (bool, ignore_bom)],
  []
);

impl StructToV8 for TextDecoder {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, encoding);
    to_v8_prop!(self, obj, scope, fatal, ());
    to_v8_prop!(self, obj, scope, ignore_bom, ());

    obj
  }
}
