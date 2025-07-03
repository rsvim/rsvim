//! Ex-commands.

use crate::prelude::*;

use compact_str::CompactString;

#[derive(Debug)]
/// Ex-command definition.
pub struct ExCommand {
  _name: CompactString,
  // backend: JsCallbackFunction,
}

impl ExCommand {
  pub fn new(name: CompactString) -> Self {
    Self { _name: name }
  }
}

#[derive(Debug)]
pub struct ExCommandsManager {
  _ex_commands: HashMap<CompactString, ExCommand>,
}

arc_mutex_ptr!(ExCommandsManager);

impl Default for ExCommandsManager {
  fn default() -> Self {
    ExCommandsManager::new()
  }
}

impl ExCommandsManager {
  pub fn new() -> Self {
    Self {
      _ex_commands: HashMap::new(),
    }
  }

  pub fn insert(&mut self, _cmd: ExCommand) -> Option<ExCommand> {
    None
  }

  pub fn remove(&mut self, _name: CompactString) -> Option<ExCommand> {
    None
  }

  pub fn get(&self, _command_line_content: CompactString) -> Option<ExCommand> {
    None
  }
}
