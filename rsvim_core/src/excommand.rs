//! Ex-commands.

use crate::prelude::*;

use compact_str::CompactString;

pub mod registry;

#[derive(Debug)]
/// Ex-command definition.
pub struct ExCommand {
  name: CompactString,
  // backend: JsCallbackFunction,
}

impl ExCommand {
  pub fn new(name: CompactString) -> Self {
    Self { name }
  }
}

#[derive(Debug)]
pub struct ExCommandsManager {
  ex_commands: HashMap<CompactString, ExCommand>,
}

arc_mutex_ptr!(ExCommandsManager);

#[derive(Debug)]
/// Ex-command instance.
pub struct ExCommandObj {
  name: CompactString,
  payload: CompactString,
}
