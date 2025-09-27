//! Ex command options.

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

impl CommandOptions {
  pub fn from_v8_object<'a>(
    scope: &mut v8::HandleScope,
    value: v8::Local<'a, v8::Object>,
  ) -> Self {
    let mut builder = CommandOptionsBuilder::default();

    // force
    let force_name = v8::String::new(scope, FORCE_NAME).unwrap();
    match value.get(scope, force_name.into()) {
      Some(force_value) => {
        let force = force_value.to_boolean(scope).boolean_value(scope);
        trace!("|from_v8_object| force:{:?}", force);
        builder.force(force);
      }
      None => { /* do nothing */ }
    }

    // alias
    let alias_name = v8::String::new(scope, ALIAS_NAME).unwrap();
    match value.get(scope, alias_name.into()) {
      Some(alias_value) => {
        let alias = alias_value.to_rust_string_lossy(scope);
        trace!("|from_v8_object| alias:{:?}", alias);
        builder.alias(Some(alias.to_compact_string()));
      }
      None => { /* do nothing */ }
    }

    builder.build().unwrap()
  }

  pub fn into_v8_object<'a>(
    &self,
    scope: &mut v8::HandleScope<'a>,
  ) -> v8::Local<'a, v8::Object> {
    let obj = v8::Object::new(scope);

    // force
    let force_field = v8::String::new(scope, "force").unwrap();
    let force_value = v8::Boolean::new(scope, self.force);
    obj.set(scope, force_field.into(), force_value.into());

    // alias
    if let Some(alias) = &self.alias {
      let alias_field = v8::String::new(scope, "alias").unwrap();
      let alias_value = v8::String::new(scope, alias).unwrap();
      obj.set(scope, alias_field.into(), alias_value.into());
    }

    obj
  }
}
