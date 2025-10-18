//! TextDecoder and its options

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::js::binding;
use crate::js::converter::*;
use compact_str::CompactString;

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
    let fatal_name = to_v8(scope, FATAL);
    if let Some(fatal_value) = value.get(scope, fatal_name) {
      builder.fatal(from_v8::<bool>(scope, fatal_value));
    }

    // ignoreBOM
    let ignore_bom_name = to_v8(scope, IGNORE_BOM);
    if let Some(ignore_bom_value) = value.get(scope, ignore_bom_name) {
      builder.ignore_bom(from_v8::<bool>(scope, ignore_bom_value));
    }

    builder.build().unwrap()
  }
}

impl ToV8 for DecoderOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // fatal
    let fatal_value = to_v8(scope, self.fatal());
    binding::set_property_to(scope, obj, FATAL, fatal_value);

    // ignoreBOM
    let ignore_bom_value = to_v8(scope, self.ignore_bom());
    binding::set_property_to(scope, obj, IGNORE_BOM, ignore_bom_value);

    obj.into()
  }
}

pub const ENCODING: &str = "encoding";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoder {
  pub options: DecoderOptions,
  pub encoding: CompactString,
}

impl FromV8 for Decoder {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    let obj = value.to_object(scope).unwrap();

    // encoding
    let encoding_name = to_v8(scope, ENCODING);
    // let encoding_name = v8::String::new(scope, ENCODING).unwrap();
    debug_assert!(obj.get(scope, encoding_name).is_some());
    let encoding_value = obj.get(scope, encoding_name).unwrap();
    let encoding_value = from_v8::<CompactString>(scope, encoding_value);

    // fatal
    let fatal_name = to_v8(scope, FATAL);
    debug_assert!(obj.get(scope, fatal_name).is_some());
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

impl ToV8 for Decoder {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // encoding
    let encoding_value = to_v8(scope, self.encoding);
    binding::set_constant_to(scope, obj, ENCODING, encoding_value);

    // fatal
    let fatal_value = to_v8(scope, self.options.fatal());
    binding::set_constant_to(scope, obj, FATAL, fatal_value);

    // ignoreBOM
    let ignore_bom_value = to_v8(scope, self.options.ignore_bom());
    binding::set_constant_to(scope, obj, IGNORE_BOM, ignore_bom_value);

    obj.into()
  }
}
