//! TextDecoder and its options

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::js::binding;
use crate::js::converter::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

flags_impl!(Flags, u8, FATAL, IGNORE_BOM);

/// Option names.
pub const FATAL: &str = "fatal";
pub const IGNORE_BOM: &str = "ignoreBOM";

/// Default option values.
pub const _FATAL_DEFAULT: bool = false;
pub const _IGNORE_BOM_DEFAULT: bool = false;

// fatal=false
// ignoreBOM=false
const FLAGS: Flags = Flags::empty();

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct DecoderOptions {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // fatal
  // ignoreBOM
  flags: Flags,
}

flags_builder_impl!(DecoderOptionsBuilder, flags, Flags, fatal, ignore_bom);

impl DecoderOptions {
  pub fn fatal(&self) -> bool {
    self.flags.contains(Flags::FATAL)
  }

  pub fn ignore_bom(&self) -> bool {
    self.flags.contains(Flags::IGNORE_BOM)
  }
}

pub trait DecoderOptionsFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Object>,
  ) -> Self;
}

impl DecoderOptionsFromV8 for DecoderOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = DecoderOptionsBuilder::default();

    // fatal
    let fatal_name = FATAL.to_v8(scope);
    if let Some(fatal_value) = value.get(scope, fatal_name) {
      builder.fatal(bool::from_v8(scope, fatal_value));
    }

    // ignoreBOM
    let ignore_bom_name = IGNORE_BOM.to_v8(scope);
    if let Some(ignore_bom_value) = value.get(scope, ignore_bom_name) {
      builder.ignore_bom(bool::from_v8(scope, ignore_bom_value));
    }

    builder.build().unwrap()
  }
}

pub trait DecoderOptionsToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object>;
}

impl DecoderOptionsToV8 for DecoderOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    // fatal
    let fatal_value = self.fatal().to_v8(scope);
    binding::set_property_to(scope, obj, FATAL, fatal_value);

    // ignoreBOM
    let ignore_bom_value = self.ignore_bom().to_v8(scope);
    binding::set_property_to(scope, obj, IGNORE_BOM, ignore_bom_value);

    obj
  }
}

pub const ENCODING: &str = "encoding";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoder {
  pub options: DecoderOptions,
  pub encoding: CompactString,
}

pub trait DecoderToV8 {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object>;
}

impl DecoderToV8 for Decoder {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    // encoding
    let encoding_value = self.encoding.to_v8(scope);
    binding::set_constant_to(scope, obj, ENCODING, encoding_value);

    // fatal
    let fatal_value = self.options.fatal().to_v8(scope);
    binding::set_constant_to(scope, obj, FATAL, fatal_value);

    // ignoreBOM
    let ignore_bom_value = self.options.ignore_bom().to_v8(scope);
    binding::set_constant_to(scope, obj, IGNORE_BOM, ignore_bom_value);

    obj
  }
}

pub trait DecoderFromV8 {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Object>,
  ) -> Self;
}

impl DecoderFromV8 for Decoder {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Object>,
  ) -> Self {
    let obj = value.to_object(scope).unwrap();

    // encoding
    let encoding_name = ENCODING.to_v8(scope);
    debug_assert!(obj.has_own_property(scope, encoding_name).unwrap_or(false));
    let encoding_value = obj.get(scope, encoding_name).unwrap();
    let encoding_value =
      String::from_v8(scope, encoding_value).to_compact_string();

    // fatal
    let fatal_name = FATAL.to_v8(scope);
    debug_assert!(obj.has_own_property(scope, fatal_name).unwrap_or(false));
    let fatal_value = obj.get(scope, fatal_name).unwrap();
    let fatal_value = from_v8::<bool>(scope, fatal_value);

    // ignoreBOM
    let ignore_bom_name = to_v8(scope, IGNORE_BOM);
    debug_assert!(obj.get(scope, ignore_bom_name).is_some());
    let ignore_bom_value = obj.get(scope, ignore_bom_name).unwrap();
    let ignore_bom_value = from_v8::<bool>(scope, ignore_bom_value);

    Self {
      options: DecoderOptionsBuilder::default()
        .fatal(fatal_value)
        .ignore_bom(ignore_bom_value)
        .build()
        .unwrap(),
      encoding: encoding_value,
    }
  }
}
