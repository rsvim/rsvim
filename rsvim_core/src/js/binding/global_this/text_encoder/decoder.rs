//! TextDecoder related.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_prop;
use crate::js::converter::*;
use compact_str::CompactString;

/// Option names.
pub const FATAL: &str = "fatal";
pub const IGNORE_BOM: &str = "ignoreBOM";
pub const ENCODING: &str = "encoding";

/// Default option values.
pub const _FATAL_DEFAULT: bool = false;
pub const _IGNORE_BOM_DEFAULT: bool = false;
pub const ENCODING_DEFAULT: CompactString = CompactString::const_new("utf-8");

flags_impl!(Flags, u8, FATAL, IGNORE_BOM);

// fatal=false
// ignoreBOM=false
const FLAGS: Flags = Flags::empty();

/// Decode option names.
pub const STREAM: &str = "stream";

/// Default decode option values.
pub const _STREAM_DEFAULT: bool = false;

flags_impl!(DecodeOptionFlags, u8, STREAM);

// stream=false
const DECODE_OPTION_FLAGS: DecodeOptionFlags = DecodeOptionFlags::empty();

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct DecodeOptions {
  #[builder(default = DECODE_OPTION_FLAGS)]
  #[builder(setter(custom))]
  // stream
  decode_option_flags: DecodeOptionFlags,
}

impl DecodeOptionsBuilder {
  flags_builder_impl!(decode_option_flags, stream);
}

impl DecodeOptions {
  pub fn stream(&self) -> bool {
    self.decode_option_flags.contains(DecodeOptionFlags::STREAM)
  }
}

impl StructFromV8 for DecodeOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = DecodeOptionsBuilder::default();

    from_v8_prop!(builder, obj, scope, bool, stream);

    builder.build().unwrap()
  }
}
