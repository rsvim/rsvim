//! TextDecoder and its options

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

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct TextDecoderOptions {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // fatal
  // ignoreBOM
  flags: Flags,
}

impl TextDecoderOptionsBuilder {
  flags_builder_impl!(flags, fatal);
  flags_builder_impl!(flags, ignore_bom);
}

impl TextDecoderOptions {
  pub fn fatal(&self) -> bool {
    self.flags.contains(Flags::FATAL)
  }

  pub fn ignore_bom(&self) -> bool {
    self.flags.contains(Flags::IGNORE_BOM)
  }
}

impl StructFromV8 for TextDecoderOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = TextDecoderOptionsBuilder::default();

    from_v8_prop!(builder, obj, scope, bool, fatal);
    from_v8_prop!(builder, obj, scope, bool, ignore_bom);

    builder.build().unwrap()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct TextDecoder {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // fatal
  // ignoreBOM
  flags: Flags,

  #[builder(default = ENCODING_DEFAULT)]
  pub encoding: CompactString,
}

impl TextDecoderBuilder {
  flags_builder_impl!(flags, fatal);
  flags_builder_impl!(flags, ignore_bom);
}

impl TextDecoder {
  pub fn fatal(&self) -> bool {
    self.flags.contains(Flags::FATAL)
  }

  #[allow(dead_code)]
  pub fn ignore_bom(&self) -> bool {
    self.flags.contains(Flags::IGNORE_BOM)
  }
}

impl StructFromV8 for TextDecoder {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = TextDecoderBuilder::default();

    from_v8_prop!(builder, obj, scope, CompactString, encoding);
    from_v8_prop!(builder, obj, scope, bool, fatal);
    from_v8_prop!(builder, obj, scope, bool, ignore_bom);

    builder.build().unwrap()
  }
}

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
