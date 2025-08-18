//! Vim ex command.

use crate::js::JsHandleId;
use crate::prelude::*;

use compact_str::{CompactString, ToCompactString};

const JS_COMMAND_NAME: &str = "js";
const JS_COMMAND_HANDLE_ID: JsHandleId = -1;

#[derive(Debug, Clone)]
/// Vim ex command instance.
pub struct ExCommand {
  name: CompactString,
  payload: CompactString,
  handle_id: JsHandleId,
  is_js: bool,
}

arc_mutex_ptr!(ExCommand);

impl ExCommand {
  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn payload(&self) -> &str {
    &self.payload
  }

  pub fn js_handle(&self) -> JsHandleId {
    self.handle_id
  }

  pub fn is_js(&self) -> bool {
    self.is_js
  }
}

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

  pub fn parse(&self, command_line: &str) -> Option<ExCommand> {
    match command_line.find(char::is_whitespace) {
      Some(pos) => {
        let name = command_line.get(0..pos).unwrap().trim().to_compact_string();
        let payload =
          command_line.get(pos..).unwrap().trim().to_compact_string();
        let is_js = name == JS_COMMAND_NAME;
        if is_js {
          let handle_id = JS_COMMAND_HANDLE_ID;
          debug_assert!(!self.command_ids.contains_key(&name));
          Some(ExCommand {
            name,
            payload,
            is_js,
            handle_id,
          })
        } else {
          match self.command_ids.get(&name) {
            Some(handle_id) => {
              let handle_id = *handle_id;
              debug_assert!(handle_id > 0);
              Some(ExCommand {
                name,
                payload,
                is_js,
                handle_id,
              })
            }
            None => None,
          }
        }
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
