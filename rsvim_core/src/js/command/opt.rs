//! Ex command options.

use crate::js::converter::*;
use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

/// Command option names.
pub const FORCE_NAME: &str = "force";
pub const ALIAS_NAME: &str = "alias";

/// Default command options.
pub const FORCE_VALUE: bool = true;
pub const ALIAS_VALUE: Option<CompactString> = None;

#[derive(Debug, Clone, derive_builder::Builder)]
pub struct CommandOptions {
  #[builder(default = FORCE_VALUE)]
  pub force: bool,

  #[builder(default = ALIAS_VALUE)]
  pub alias: Option<CompactString>,
}

impl FromV8 for CommandOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Option<Self> {
    let mut builder = CommandOptionsBuilder::default();

    // force
    let force_name = to_v8(scope, FORCE_NAME).unwrap();
    if let Some(force_value) = value.get(scope, force_name) {
      builder.force(from_v8::<bool>(scope, force_value).unwrap());
    }

    // alias
    let alias_name = to_v8(scope, ALIAS_NAME).unwrap();
    if let Some(alias_value) = value.get(scope, alias_name) {
      builder
        .alias(Some(from_v8::<CompactString>(scope, alias_value).unwrap()));
    }

    builder.build().unwrap()
  }
}

impl ToV8 for CommandOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
    let obj = v8::Object::new(scope);

    // force
    let force_field = to_v8(scope, "force").unwrap();
    let force_value = to_v8(scope, self.force).unwrap();
    obj.set(scope, force_field.into(), force_value.into());

    // alias
    if let Some(alias) = &self.alias {
      let alias_field = to_v8(scope, "alias").unwrap();
      let alias_value = to_v8(scope, alias).unwrap();
      obj.set(scope, alias_field.into(), alias_value.into());
    }

    obj.into()
  }
}
