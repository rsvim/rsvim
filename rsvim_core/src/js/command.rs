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
use std::rc::Rc;

const JS_COMMAND_NAME: &str = "js";
pub type ExCommandCallback = Rc<v8::Global<v8::Function>>;

#[derive(Debug, Clone)]
/// Ex command execution instance
pub struct BuiltinExCommandFuture {
  pub task_id: JsTaskId,
  pub name: CompactString,
  pub body: CompactString,
}

impl JsFuture for BuiltinExCommandFuture {
  fn run(&mut self, scope: &mut v8::HandleScope) {
    trace!("|BuiltinExCommandFuture| run:{:?}", self.task_id);
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

#[derive(Debug, Clone)]
/// Ex command execution instance
pub struct UserExCommandFuture {
  pub task_id: JsTaskId,
  pub name: CompactString,
  pub cb: ExCommandCallback,
}

#[derive(Debug, Default)]
pub struct ExCommandsManager {
  commands: FoldMap<CompactString, ExCommandCallback>,
}

arc_mutex_ptr!(ExCommandsManager);

pub type ExCommandsManagerKeys<'a> =
  std::collections::hash_map::Keys<'a, CompactString, ExCommandCallback>;
pub type ExCommandsManagerValues<'a> =
  std::collections::hash_map::Values<'a, CompactString, ExCommandCallback>;
pub type ExCommandsManagerIter<'a> =
  std::collections::hash_map::Iter<'a, CompactString, ExCommandCallback>;

impl ExCommandsManager {
  pub fn is_empty(&self) -> bool {
    self.commands.is_empty()
  }

  pub fn len(&self) -> usize {
    self.commands.len()
  }

  pub fn remove(&mut self, name: &str) -> Option<ExCommandCallback> {
    self.commands.remove(name)
  }

  pub fn insert(
    &mut self,
    name: CompactString,
    cb: Rc<v8::Global<v8::Function>>,
  ) -> Option<ExCommandCallback> {
    self.commands.insert(name, cb)
  }

  pub fn get(&self, name: &str) -> Option<ExCommandCallback> {
    self.commands.get(name).cloned()
  }

  pub fn contains_key(&self, name: &str) -> bool {
    self.commands.contains_key(name)
  }

  pub fn keys(&self) -> ExCommandsManagerKeys<'_> {
    self.commands.keys()
  }

  pub fn values(&self) -> ExCommandsManagerValues<'_> {
    self.commands.values()
  }

  pub fn iter(&self) -> ExCommandsManagerIter<'_> {
    self.commands.iter()
  }
}

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
      })
    } else if self.commands.contains_key(&name) {
      Some(BuiltinExCommandFuture {
        task_id,
        name,
        body,
      })
    } else {
      None
    }
  }
}
