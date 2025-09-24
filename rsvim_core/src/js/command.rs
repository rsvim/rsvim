//! Vim ex commands.

use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::JsTaskId;
use crate::js::binding;
use crate::js::execute_module;
use crate::js::next_task_id;
use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

const JS_COMMAND_NAME: &str = "js";

#[derive(Debug, Clone)]
/// Ex command execution instance
pub struct BuiltinExCommandFuture {
  pub task_id: JsTaskId,
  pub name: CompactString,
  pub body: CompactString,
  pub is_builtin_js: bool,
}

impl JsFuture for BuiltinExCommandFuture {
  fn run(&mut self, scope: &mut v8::HandleScope) {
    trace!("|ExCommand| run:{:?}", self.task_id);
    debug_assert!(self.is_builtin_js);
    let filename = format!("<command{}>", self.task_id);

    match execute_module(scope, &filename, Some(self.body.trim())) {
      Ok(_) => { /* do nothing */ }
      Err(e) => {
        // Capture exception if there's any error while loading/evaluating module.
        trace!("Failed to execute module, filename:{filename:?}, error:{e:?}");
        let message = v8::String::new(scope, &e.to_string()).unwrap();
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

pub type JsCommand = (v8::Global<v8::Function>, Vec<v8::Global<v8::Value>>);

#[derive(Debug, Default)]
pub struct ExCommandsManager {
  commands: FoldMap<CompactString, JsCommand>,
}

arc_mutex_ptr!(ExCommandsManager);

impl ExCommandsManager {
  pub fn parse(&self, payload: &str) -> Option<BuiltinExCommandFuture> {
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
    let task_id = next_task_id();
    if is_builtin_js {
      debug_assert!(!self.commands.contains_key(&name));
      Some(BuiltinExCommandFuture {
        task_id,
        name,
        body,
        is_builtin_js,
      })
    } else if self.commands.contains_key(&name) {
      Some(BuiltinExCommandFuture {
        task_id,
        name,
        body,
        is_builtin_js,
      })
    } else {
      None
    }
  }
}
