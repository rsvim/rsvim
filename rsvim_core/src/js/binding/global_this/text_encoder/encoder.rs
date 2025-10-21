//! TextEncoder.

use crate::js::converter::*;
use crate::to_v8_prop;

/// Option names.
pub const ENCODING: &str = "encoding";

pub const _ENCODING_DEFAULT: &str = "utf-8";

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct TextEncoder {
  pub encoding: String,
}

impl StructToV8 for TextEncoder {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, encoding);

    obj
  }
}
