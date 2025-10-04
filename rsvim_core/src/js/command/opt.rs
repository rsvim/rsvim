//! Ex command options.

use crate::js::converter::*;
use bitflags::bitflags;
use compact_str::CompactString;
use std::fmt::Debug;

/// Command option names.
pub const FORCE: &str = "force";
pub const ALIAS: &str = "alias";

/// Default command options.
pub const FORCE_DEFAULT: bool = true;
pub const ALIAS_DEFAULT: Option<CompactString> = None;

bitflags! {
  #[derive(Copy, Clone, PartialEq, Eq)]
  struct Flags: u8 {
    const FORCE = 1;
  }
}

impl Debug for Flags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Flags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

// force=true
const FLAGS: Flags = Flags::all();

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandOptions {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // force=true
  flags: Flags,

  #[builder(default = ALIAS_DEFAULT)]
  alias: Option<CompactString>,
}

impl CommandOptionsBuilder {
  pub fn force(&mut self, value: bool) -> &mut Self {
    let mut flags = match self.flags {
      Some(flags) => flags,
      None => FLAGS,
    };
    if value {
      flags.insert(Flags::FORCE);
    } else {
      flags.remove(Flags::FORCE);
    }
    self.flags = Some(flags);
    self
  }
}

impl CommandOptions {
  pub fn force(&self) -> bool {
    self.flags.contains(Flags::FORCE)
  }

  pub fn set_force(&mut self, value: bool) {
    if value {
      self.flags.insert(Flags::FORCE);
    } else {
      self.flags.remove(Flags::FORCE);
    }
  }

  pub fn alias(&self) -> &Option<CompactString> {
    &self.alias
  }

  pub fn set_alias(&mut self, value: Option<CompactString>) {
    self.alias = value;
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
