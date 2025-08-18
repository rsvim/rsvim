//! Vim ex command.

use crate::js::JsHandleId;
use crate::prelude::*;

use compact_str::{CompactString, ToCompactString};

pub mod parser;

#[derive(Debug)]
pub struct ExCommandsManager {
  command_ids: HashMap<CompactString, JsHandleId>,
}

arc_mutex_ptr!(ExCommandsManager);

impl ExCommandsManager {
  pub fn new() -> Self {
    Self {
      command_ids: HashMap::new(),
    }
  }

  pub fn get(&self, name: &str) -> Option<&JsHandleId> {
    self.command_ids.get(name)
  }

  pub fn parse(
    &self,
    command_line: &str,
  ) -> Option<(&CompactString, &JsHandleId)> {
    match command_line.find(char::is_whitespace) {
      Some(pos) => {
        let name = command_line.get(0..pos).unwrap().trim();
        self.command_ids.get_key_value(name)
      }
      None => None,
    }
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
