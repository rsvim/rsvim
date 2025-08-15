//! Vim ex command.
//!
//! NOTE: For the word "ex command" in rsvim, it is more used to describe a
//! product feature: When user types ":" in normal mode, starts command-line
//! mode and input commands. The "ex" word is used to distinguish it from
//! searching forward/backward in command line mode.

use crate::js::JsFutureId;
use crate::prelude::*;

use compact_str::CompactString;

#[derive(Debug, PartialEq, Eq)]
/// Ex-command definition.
pub struct ExCommand {
  // Command name, case sensitive
  name: CompactString,

  // For all commands, they are implemented/registered with a javascript
  // callback function, managed by js runtime.
  js_callback_id: JsFutureId,
}

impl ExCommand {
  pub fn new(name: CompactString, js_callback_id: JsFutureId) -> Self {
    Self {
      name,
      js_callback_id,
    }
  }
}

#[derive(Debug)]
pub struct ExCommandsManager {
  ex_commands: HashMap<CompactString, ExCommand>,
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
      ex_commands: HashMap::new(),
    }
  }

  pub fn insert(&mut self, _cmd: ExCommand) -> Option<ExCommand> {
    None
  }

  pub fn remove(&mut self, _name: CompactString) -> Option<ExCommand> {
    None
  }

  pub fn get(&self, _name: CompactString) -> Option<ExCommand> {
    None
  }
}
