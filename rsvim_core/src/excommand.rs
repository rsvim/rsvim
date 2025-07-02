//! Ex-commands.

use crate::prelude::*;

use compact_str::CompactString;

pub mod registry;

#[derive(Debug)]
pub struct ExCommand {
  name: CompactString,
  // backend: JsCallbackFunction,
}

arc_mutex_ptr!(ExCommand);

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
