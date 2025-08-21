//! Vim ex commands.

use crate::js::{self, JsFuture, JsFutureId, execute_module_impl};
use crate::prelude::*;

use compact_str::{CompactString, ToCompactString};

const JS_COMMAND_NAME: &str = "js";

#[derive(Debug, Clone)]
/// Parsed ex command instance
pub struct ExCommand {
  future_id: JsFutureId,
  name: CompactString,
  payload: CompactString,
  is_js: bool,
}

impl JsFuture for ExCommand {
  fn run(&mut self, scope: &mut v8::HandleScope) {
    debug_assert!(self.command.is_js());
    let filename = format!("<ExCommand{}>", self.future_id);
    execute_module_impl(scope, &filename, Some(self.command.payload()))
      .unwrap();
  }
}

#[derive(Debug)]
pub struct ExCommandManager {
  commands: HashSet<CompactString>,
}

arc_mutex_ptr!(ExCommandManager);

impl ExCommandManager {
  pub fn new() -> Self {
    Self {
      commands: HashSet::new(),
    }
  }

  pub fn parse(&self, payload: &str) -> Option<ExCommand> {
    match payload.find(char::is_whitespace) {
      Some(pos) => {
        let name = payload.get(0..pos).unwrap().trim().to_compact_string();
        let payload = payload.get(pos..).unwrap().to_compact_string();
        let is_js = name == JS_COMMAND_NAME;
        let command_id = js::next_future_id();
        if is_js {
          debug_assert!(!self.commands.contains(&name));
          Some(ExCommand {
            future_id: command_id,
            name,
            payload,
            is_js,
          })
        } else {
          if self.commands.contains(&name) {
            Some(ExCommand {
              future_id: command_id,
              name,
              payload,
              is_js,
            })
          } else {
            None
          }
        }
      }
      None => None,
    }
  }
}

impl Default for ExCommandManager {
  fn default() -> Self {
    Self::new()
  }
}
