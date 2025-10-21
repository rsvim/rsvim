//! Ex command options.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_impl;
use crate::js::converter::*;
use crate::to_v8_opt_prop;
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

flags_builder_impl!(CommandOptions, flags, force);

impl CommandOptions {
  pub fn force(&self) -> bool {
    self.flags.contains(Flags::FORCE)
  }
}

from_v8_impl!(CommandOptions, [(bool, force)], [(CompactString, alias)]);

// to_v8_impl!(CommandOptions, [force], [alias], []);

impl StructToV8 for CommandOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, force, ());
    to_v8_opt_prop!(self, obj, scope, alias);

    obj
  }
}
