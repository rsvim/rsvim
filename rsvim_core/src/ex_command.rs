//! Vim ex command.

use crate::js::JsHandleId;
use crate::prelude::*;

use compact_str::{CompactString, ToCompactString};

pub mod parser;

pub struct ExCommandsManager {
  command_ids: HashMap<CompactString, JsHandleId>,
}

impl ExCommandsManager {
  pub fn new() -> Self {
    Self {
      command_ids: HashMap::new(),
    }
  }

  pub fn get_command_id(&self, name: &str) -> Option<&JsHandleId> {
    self.command_ids.get(name)
  }

  pub fn create_command(
    &mut self,
    name: &str,
    command_id: JsHandleId,
  ) -> Option<JsHandleId> {
    self
      .command_ids
      .insert(name.to_compact_string(), command_id)
  }

  pub fn remove_command(&mut self, name: &str) -> Option<JsHandleId> {
    self.command_ids.remove(name)
  }
}
