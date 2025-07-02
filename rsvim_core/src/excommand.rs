//! Ex-commands.

use crate::prelude::*;

use compact_str::CompactString;

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

impl ExCommandsManager {
  pub fn new() -> Self {
    Self {
      ex_commands: HashMap::new(),
    }
  }

  pub fn insert(&mut self, cmd: ExCommand) -> Option<ExCommand> {
    None
  }

  pub fn remove(&mut self, name: CompactString) -> Option<ExCommand> {
    None
  }

  pub fn get(&self, command_line_content: CompactString) -> Option<ExCommand> {
    None
  }
}
