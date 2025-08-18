//! Vim ex command.

use crate::js::JsHandleId;
use crate::prelude::*;

use compact_str::{CompactString, ToCompactString};

pub mod parser;

#[derive(Debug)]
pub struct ExCommandsManager {
  command_ids: HashMap<CompactString, JsHandleId>,
}

impl ExCommandsManager {
  pub fn new() -> Self {
    Self {
      command_ids: HashMap::new(),
    }
  }

  pub fn get(&self, name: &str) -> Option<&JsHandleId> {
    self.command_ids.get(name)
  }

  pub fn insert(
    &mut self,
    name: &str,
    command_id: JsHandleId,
  ) -> Option<JsHandleId> {
    self
      .command_ids
      .insert(name.to_compact_string(), command_id)
  }

  pub fn remove(&mut self, name: &str) -> Option<JsHandleId> {
    self.command_ids.remove(name)
  }
}

impl Default for ExCommandsManager {
  fn default() -> Self {
    Self::new()
  }
}
