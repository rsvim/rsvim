//! Ex command options.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_prop;
use crate::js::converter::*;
use crate::to_v8_prop;
use compact_str::CompactString;

flags_impl!(Flags, u8, FORCE);

/// Command option names.
pub const FORCE: &str = "force";
pub const ALIAS: &str = "alias";

/// Default command options.
pub const FORCE_DEFAULT: bool = true;
pub const ALIAS_DEFAULT: Option<CompactString> = None;

// force=true
const FLAGS: Flags = Flags::FORCE;

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandOptions {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // force
  flags: Flags,

  #[builder(default = ALIAS_DEFAULT)]
  pub alias: Option<CompactString>,
}

impl CommandOptionsBuilder {
  flags_builder_impl!(flags, force);
}

impl CommandOptions {
  pub fn force(&self) -> bool {
    self.flags.contains(Flags::FORCE)
  }
}

impl StructFromV8 for CommandOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = CommandOptionsBuilder::default();

    from_v8_prop!(builder, obj, scope, bool, force);
    from_v8_prop!(builder, obj, scope, CompactString, alias, optional);

    builder.build().unwrap()
  }
}

impl StructToV8 for CommandOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, force, ());
    to_v8_prop!(self, obj, scope, alias, optional);

    obj
  }
}
