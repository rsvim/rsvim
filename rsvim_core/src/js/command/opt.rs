//! Ex command options.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::js::converter::*;
use compact_str::CompactString;

flags_impl!(Flags, u8, FORCE, 0b0000_0001);

/// Command option names.
pub const FORCE: &str = "force";
pub const ALIAS: &str = "alias";

/// Default command options.
pub const FORCE_DEFAULT: bool = true;
pub const ALIAS_DEFAULT: Option<CompactString> = None;

const FLAGS: Flags = Flags::FORCE;

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandOptions {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // force=true
  flags: Flags,

  #[builder(default = ALIAS_DEFAULT)]
  pub alias: Option<CompactString>,
}

flags_builder_impl!(CommandOptionsBuilder, flags, FLAGS, force, Flags::FORCE);

impl CommandOptions {
  pub fn force(&self) -> bool {
    self.flags.contains(Flags::FORCE)
  }
}

impl FromV8 for CommandOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    let mut builder = CommandOptionsBuilder::default();
    let obj = value.to_object(scope).unwrap();

    // force
    let force = to_v8(scope, FORCE);
    if let Some(force_value) = obj.get(scope, force) {
      builder.force(from_v8::<bool>(scope, force_value));
    }

    // alias
    let alias = to_v8(scope, ALIAS);
    if let Some(has_alias) = obj.has(scope, alias) {
      if has_alias {
        if let Some(alias_value) = obj.get(scope, alias) {
          builder.alias(Some(from_v8::<CompactString>(scope, alias_value)));
        }
      }
    }

    builder.build().unwrap()
  }
}

impl ToV8 for CommandOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // force
    let force_field = to_v8(scope, FORCE);
    let force_value = to_v8(scope, self.force());
    obj.set(scope, force_field, force_value);

    // alias
    if let Some(alias) = &self.alias {
      let alias_field = to_v8(scope, ALIAS);
      let alias_value = to_v8(scope, alias.clone());
      obj.set(scope, alias_field, alias_value);
    }

    obj.into()
  }
}
