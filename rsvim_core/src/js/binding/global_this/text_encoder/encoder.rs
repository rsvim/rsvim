//! TextEncoder.

use crate::from_v8_prop;
use crate::js::converter::*;
use crate::to_v8_const;
use compact_str::CompactString;

/// Option names.
pub const ENCODING: &str = "encoding";

pub const ENCODING_DEFAULT: CompactString = CompactString::const_new("utf-8");

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct TextEncoder {
  #[builder(default = ENCODING_DEFAULT)]
  pub encoding: CompactString,
}

impl StructFromV8 for TextEncoder {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = TextEncoderBuilder::default();

    from_v8_prop!(builder, obj, scope, CompactString, encoding);

    builder.build().unwrap()
  }
}

impl StructToV8 for TextEncoder {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_const!(self, obj, scope, encoding);

    obj
  }
}
