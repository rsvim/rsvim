//! Vim ex commands.

use crate::js;
use crate::js::JsFuture;
use crate::js::JsFutureId;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::execute_module;
use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

const JS_COMMAND_NAME: &str = "js";

#[derive(Debug, Clone)]
/// Ex command execution instance
pub struct ExCommand {
  pub future_id: JsFutureId,
  pub name: CompactString,
  pub body: CompactString,
  pub is_builtin_js: bool,
}

impl JsFuture for ExCommand {
  fn run(&mut self, scope: &mut v8::HandleScope) {
    debug_assert!(self.is_builtin_js);
    let filename = format!("<command{}>", self.future_id);

    match execute_module(scope, &filename, Some(self.body().trim())) {
      Ok(_) => { /* do nothing */ }
      Err(e) => {
        // Capture exception if there's any error while loading/evaluating module.
        trace!("Failed to execute module, filename:{filename:?}, error:{e:?}");
        let message = e.to_string().to_owned();
        let message = v8::String::new(scope, &message).unwrap();
        let exception = v8::Exception::error(scope, message);
        binding::set_exception_code(scope, exception, &e);
        let exception = v8::Global::new(scope, exception);
        let state_rc = JsRuntime::state(scope);
        state_rc
          .borrow_mut()
          .exceptions
          .capture_exception(exception);
      }
    }
  }
}

#[derive(Debug, Default)]
pub struct ExCommandsManager {
  commands: HashSet<CompactString>,
}

arc_mutex_ptr!(ExCommandsManager);

impl ExCommandsManager {
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
