//! Vim ex commands.

use crate::js::{self, JsFuture, JsFutureId, JsRuntime, execute_module_impl};
use crate::prelude::*;

use compact_str::{CompactString, ToCompactString};

const JS_COMMAND_NAME: &str = "js";

#[derive(Debug, Clone)]
/// Ex command execution instance
pub struct ExCommand {
  future_id: JsFutureId,
  name: CompactString,
  body: CompactString,
  is_builtin_js: bool,
}

impl ExCommand {
  pub fn future_id(&self) -> JsFutureId {
    self.future_id
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn body(&self) -> &str {
    &self.body
  }

  pub fn is_builtin_js(&self) -> bool {
    self.is_builtin_js
  }
}

impl JsFuture for ExCommand {
  fn run(&mut self, scope: &mut v8::HandleScope) {
    // For now only `:js` command is supported.
    debug_assert!(self.is_builtin_js());
    let filename = format!("<ExCommand{}>", self.future_id);

    let tc_scope = &mut v8::TryCatch::new(scope);

    match execute_module_impl(tc_scope, &filename, Some(self.body().trim())) {
      Ok(_) => { /* do nothing */ }
      Err(e) => {
        // Report if there's any error while loading/evaluating module.
        let exception = tc_scope.exception().unwrap();
        let exception = v8::Global::new(tc_scope, exception);
        let state_rc = JsRuntime::state(tc_scope);
        state_rc
          .borrow_mut()
          .exceptions
          .capture_exception(exception);
      }
    }

    // Report if callback threw an exception.
    if tc_scope.has_caught() {
      let exception = tc_scope.exception().unwrap();
      let exception = v8::Global::new(tc_scope, exception);
      let state_rc = JsRuntime::state(tc_scope);
      state_rc
        .borrow_mut()
        .exceptions
        .capture_exception(exception);
    }
  }
}

#[derive(Debug)]
pub struct ExCommandsManager {
  commands: HashSet<CompactString>,
}

arc_mutex_ptr!(ExCommandsManager);

impl ExCommandsManager {
  pub fn new() -> Self {
    Self {
      commands: HashSet::new(),
    }
  }

  pub fn parse(&self, payload: &str) -> Option<ExCommand> {
    let (name, body) = match payload.find(char::is_whitespace) {
      Some(pos) => {
        let name = payload.get(0..pos).unwrap().trim().to_compact_string();
        let body = payload.get(pos..).unwrap().to_compact_string();
        (name, body)
      }
      None => {
        let name = payload.trim().to_compact_string();
        let body = "".to_compact_string();
        (name, body)
      }
    };

    let is_builtin_js = name == JS_COMMAND_NAME;
    let future_id = js::next_future_id();
    if is_builtin_js {
      debug_assert!(!self.commands.contains(&name));
      Some(ExCommand {
        future_id,
        name,
        body,
        is_builtin_js,
      })
    } else if self.commands.contains(&name) {
      Some(ExCommand {
        future_id,
        name,
        body,
        is_builtin_js,
      })
    } else {
      None
    }
  }
}

impl Default for ExCommandsManager {
  fn default() -> Self {
    Self::new()
  }
}
