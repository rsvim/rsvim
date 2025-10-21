//! Ex command runtime context.

use crate::js::converter::*;
use crate::to_v8_impl;
use compact_str::CompactString;

/// Command attribute name.
pub const BANG: &str = "bang";
pub const ARGS: &str = "args";

/// Default command attributes.
pub const BANG_DEFAULT: bool = false;
pub const ARGS_DEFAULT: Vec<CompactString> = vec![];

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandContext {
  #[builder(default = BANG_DEFAULT)]
  // bang
  bang: bool,

  #[builder(default = ARGS_DEFAULT)]
  args: Vec<CompactString>,
}

impl CommandContext {
  pub fn bang(&self) -> bool {
    self.bang
  }

  pub fn args(&self) -> &Vec<CompactString> {
    &self.args
  }
}

to_v8_impl!(CommandContext, [bang], [], [args]);
