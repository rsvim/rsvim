//! Vim ex commands.

pub mod attr;
pub mod def;
pub mod opt;

#[cfg(test)]
mod attr_tests;

use crate::js::JsFuture;
use crate::js::JsTaskId;
use crate::js::execute_module;
use crate::js::next_task_id;
use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use def::CommandDefinition;

const JS_COMMAND_NAME: &str = "js";

#[derive(Debug, Clone)]
/// Builtin `:js` command
pub struct BuiltinCommandFuture {
  pub task_id: JsTaskId,
  pub name: CompactString,
  pub body: CompactString,
}

impl JsFuture for BuiltinCommandFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|BuiltinCommandFuture| run:{:?}", self.task_id);
    let filename = format!("<command-js:{}>", self.task_id);

    execute_module(scope, &filename, Some(self.body.trim()));
  }
}

#[derive(Debug, Clone)]
/// User command
pub struct UserCommandFuture {
  pub task_id: JsTaskId,
  pub name: CompactString,
  pub definition: CommandDefinition,
}

#[derive(Debug, Default)]
pub struct CommandsManager {
  // Maps from command "name" to its "definition".
  commands: BTreeMap<CompactString, CommandDefinition>,

  // Maps from "alias" to its "name".
  aliases: FoldMap<CompactString, CompactString>,
}

arc_mutex_ptr!(CommandsManager);

impl CommandsManager {
  pub fn is_empty(&self) -> bool {
    self.commands.is_empty()
  }

  pub fn len(&self) -> usize {
    self.commands.len()
  }

  pub fn remove(&mut self, name: &str) -> Option<CommandDefinition> {
    self.commands.remove(name)
  }

  pub fn insert(
    &mut self,
    name: CompactString,
    definition: CommandDefinition,
  ) -> Option<CommandDefinition> {
    let new_alias = definition.options.alias.clone();

    // - Inserts new command definition by name
    // - Also removes the old command definition
    // - Then removes the old command alias if exists
    let old = self.commands.insert(name.clone(), definition);
    if let Some(ref old) = old {
      if let Some(old_alias) = &old.options.alias {
        self.aliases.remove(old_alias.as_str());
      }
    }

    // - Inserts new command alias.
    if let Some(new_alias) = new_alias {
      self.aliases.insert(new_alias.clone(), name.clone());
    }
    old
  }

  pub fn get(&self, name: &str) -> Option<CommandDefinition> {
    self.commands.get(name).cloned()
  }

  pub fn contains_key(&self, name: &str) -> bool {
    self.commands.contains_key(name)
  }

  pub fn keys(
    &self,
  ) -> std::collections::btree_map::Keys<'_, CompactString, CommandDefinition>
  {
    self.commands.keys()
  }

  pub fn values(
    &self,
  ) -> std::collections::btree_map::Values<'_, CompactString, CommandDefinition>
  {
    self.commands.values()
  }

  pub fn iter(
    &self,
  ) -> std::collections::btree_map::Iter<'_, CompactString, CommandDefinition>
  {
    self.commands.iter()
  }

  pub fn first_key_value(
    &self,
  ) -> Option<(&CompactString, &CommandDefinition)> {
    self.commands.first_key_value()
  }

  pub fn last_key_value(&self) -> Option<(&CompactString, &CommandDefinition)> {
    self.commands.last_key_value()
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
