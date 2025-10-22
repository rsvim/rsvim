//! TextEncoder related.

use crate::js::converter::*;
use crate::to_v8_prop;

/// `encodeInto` returned object field names.
pub const READ: &str = "read";
pub const WRITTEN: &str = "written";

/// Default option values.
pub const READ_DEFAULT: u32 = 0;
pub const WRITTEN_DEFAULT: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct EncodeIntoResult {
  #[builder(default = READ_DEFAULT)]
  pub read: u32,

  #[builder(default = WRITTEN_DEFAULT)]
  pub written: u32,
}

impl StructToV8 for EncodeIntoResult {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, read);
    to_v8_prop!(self, obj, scope, written);

    obj
  }
}
