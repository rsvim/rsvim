//! Ex command options.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_impl;
use crate::js::converter::*;
use crate::to_v8_impl;
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
  alias: Option<CompactString>,
}

flags_builder_impl!(CommandOptionsBuilder, flags, Flags, force);

impl CommandOptions {
  pub fn force(&self) -> bool {
    self.flags.contains(Flags::FORCE)
  }

  pub fn alias(&self) -> &Option<CompactString> {
    &self.alias
  }
}

from_v8_impl!(CommandOptions, [force], [alias]);
to_v8_impl!(CommandOptions, [force], [alias], [], []);
