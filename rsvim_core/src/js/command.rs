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
use def::CommandDefinitionRc;

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
  pub definition: CommandDefinitionRc,
}

#[derive(Debug, Default)]
pub struct CommandsManager {
  // Maps from command "name" or "alias" to its "definition".
  commands: BTreeMap<CompactString, CommandDefinitionRc>,
}

arc_mutex_ptr!(CommandsManager);

pub type CommandsManagerKeys<'a> =
  std::collections::btree_map::Keys<'a, CompactString, CommandDefinitionRc>;
pub type CommandsManagerValues<'a> =
  std::collections::btree_map::Values<'a, CompactString, CommandDefinitionRc>;
pub type CommandsManagerIter<'a> =
  std::collections::btree_map::Iter<'a, CompactString, CommandDefinitionRc>;

impl CommandsManager {
  pub fn is_empty(&self) -> bool {
    self.commands.is_empty()
  }

  pub fn len(&self) -> usize {
    self.commands.len()
  }

  pub fn remove(&mut self, name: &str) -> Option<CommandDefinitionRc> {
    self.commands.remove(name)
  }

  /// Insert new command definition.
  ///
  /// Every "command" has a unique name and it alias (if exists). When
  /// inserts/registers a new command, both its name and alias cannot conflict
  /// with existing registered ones.
  ///
  /// # Returns
  ///
  /// 1. It returns `Ok(None)` if registered successfully, and no conflicting
  ///    one exists.
  /// 2. It returns `Ok(CommandDefinition)` if registered successfully, and
  ///    previous one is been removed and returned. Note: this requires the
  ///    `force` option.
  /// 3. It returns `Err` if registered failed, because either command name or
  ///    alias already exists, and user doesn't have the `force` option.
  pub fn insert(
    &mut self,
    name: CompactString,
    definition: CommandDefinitionRc,
  ) -> AnyResult<Option<CommandDefinitionRc>> {
    let alias = definition.borrow().options.alias.clone();

    if !definition.borrow().options.force {
      if self.commands.contains_key(&name) {
        anyhow::bail!(format!("Command name {:?} already exists", name));
      }
      if let Some(ref alias) = alias {
        if self.commands.contains_key(alias.as_str()) {
          anyhow::bail!(format!("Command alias {:?} already exists", name));
        }
      }
    }

    if let Some(alias) = alias {
      self.commands.insert(alias, definition.clone());
    }
    let old = self.commands.insert(name.clone(), definition.clone());

    Ok(old)
  }

  pub fn get(&self, name: &str) -> Option<CommandDefinitionRc> {
    self.commands.get(name).cloned()
  }

  pub fn contains_key(&self, name: &str) -> bool {
    self.commands.contains_key(name)
  }

  pub fn keys(&self) -> CommandsManagerKeys {
    self.commands.keys()
  }

  pub fn values(&self) -> CommandsManagerValues {
    self.commands.values()
  }

  pub fn iter(&self) -> CommandsManagerIter {
    self.commands.iter()
  }

  pub fn first_key_value(
    &self,
  ) -> Option<(&CompactString, &CommandDefinitionRc)> {
    self.commands.first_key_value()
  }

  pub fn last_key_value(
    &self,
  ) -> Option<(&CompactString, &CommandDefinitionRc)> {
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
