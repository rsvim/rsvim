//! Vim ex commands.

pub mod attr;

use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::JsTaskId;
use crate::js::binding;
use crate::js::execute_module;
use crate::js::next_task_id;
use crate::prelude::*;
use crate::state::mode::Modes;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::rc::Rc;

const JS_COMMAND_NAME: &str = "js";

pub const BANG: &str = "bang";
pub const MODS: &str = "mods";
pub const NARGS: &str = "nargs";

#[derive(Debug, Clone)]
pub struct CommandAttributes {
  pub bang: bool,
  pub mods: Modes,
  pub nargs: 
}

pub type CommandCallback = Rc<v8::Global<v8::Function>>;

#[derive(Debug, Clone)]
/// Builtin `:js` command
pub struct BuiltinCommandFuture {
  pub task_id: JsTaskId,
  pub name: CompactString,
  pub body: CompactString,
}

impl JsFuture for BuiltinCommandFuture {
  fn run(&mut self, scope: &mut v8::HandleScope) {
    trace!("|BuiltinCommandFuture| run:{:?}", self.task_id);
    let filename = format!("<command-js:{}>", self.task_id);

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
/// User command
pub struct UserCommandFuture {
  pub task_id: JsTaskId,
  pub name: CompactString,
  pub cb: CommandCallback,
}

#[derive(Debug, Default)]
pub struct CommandsManager {
  commands: FoldMap<CompactString, (CommandCallback)>,
}

arc_mutex_ptr!(CommandsManager);

impl CommandsManager {
  pub fn is_empty(&self) -> bool {
    self.commands.is_empty()
  }

  pub fn len(&self) -> usize {
    self.commands.len()
  }

  pub fn remove(&mut self, name: &str) -> Option<CommandCallback> {
    self.commands.remove(name)
  }

  pub fn insert(
    &mut self,
    name: CompactString,
    cb: Rc<v8::Global<v8::Function>>,
  ) -> Option<CommandCallback> {
    self.commands.insert(name, cb)
  }

  pub fn get(&self, name: &str) -> Option<CommandCallback> {
    self.commands.get(name).cloned()
  }

  pub fn contains_key(&self, name: &str) -> bool {
    self.commands.contains_key(name)
  }

  pub fn keys(
    &self,
  ) -> std::collections::hash_map::Keys<'_, CompactString, CommandCallback> {
    self.commands.keys()
  }

  pub fn values(
    &self,
  ) -> std::collections::hash_map::Values<'_, CompactString, CommandCallback>
  {
    self.commands.values()
  }

  pub fn iter(
    &self,
  ) -> std::collections::hash_map::Iter<'_, CompactString, CommandCallback> {
    self.commands.iter()
  }
}

impl CommandsManager {
  pub fn parse(&self, payload: &str) -> Option<BuiltinCommandFuture> {
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
      Some(BuiltinCommandFuture {
        task_id,
        name,
        body,
      })
    } else if self.commands.contains_key(&name) {
      Some(BuiltinCommandFuture {
        task_id,
        name,
        body,
      })
    } else {
      None
    }
  }
}
