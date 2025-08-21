//! Vim ex commands.

use crate::js::{JsFuture, JsFutureId, execute_module_impl};
use crate::prelude::*;

use compact_str::CompactString;

pub struct ExCommand {
  future_id: JsFutureId,
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

  pub fn parse(&self, payload: CompactString) -> ExCommand {}
}

impl Default for ExCommandManager {
  fn default() -> Self {
    Self::new()
  }
}
