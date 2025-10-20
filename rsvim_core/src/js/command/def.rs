//! Ex command definition.

use crate::js::command::attr::*;
use crate::js::command::opt::*;
use crate::js::converter::*;
use crate::prelude::*;
use crate::to_v8_impl;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::fmt::Debug;
use std::rc::Rc;

pub type CommandCallback = Rc<v8::Global<v8::Function>>;

/// Command definition names.
pub const NAME: &str = "name";
pub const CALLBACK: &str = "callback";
pub const ATTRIBUTES: &str = "attributes";
pub const OPTIONS: &str = "options";

#[derive(Clone)]
pub struct CommandDefinition {
  pub name: CompactString,
  pub callback: CommandCallback,
  pub attributes: CommandAttributes,
  pub options: CommandOptions,
}

rc_ptr!(CommandDefinition);

impl Debug for CommandDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CommandDefinition")
      .field(NAME, &self.name)
      .field(CALLBACK, &"Rc<v8::Global<v8::Function>>")
      .field(ATTRIBUTES, &self.attributes)
      .field(OPTIONS, &self.options)
      .finish()
  }
}

impl StructFromV8CallbackArguments for CommandDefinition {
  fn from_v8_callback_arguments<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
  ) -> Self {
    debug_assert!(args.length() == 4);
    debug_assert!(args.get(0).is_string() || args.get(0).is_string_object());
    let name = String::from_v8(scope, args.get(0).to_string(scope).unwrap());
    let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let callback = Rc::new(v8::Global::new(scope, callback));
    let attributes = CommandAttributes::from_v8(scope, args.get(2));
    let options = CommandOptions::from_v8(scope, args.get(3));

    Self {
      name: name.to_compact_string(),
      callback,
      attributes,
      options,
    }
  }
}

to_v8_impl!(
  CommandDefinition,
  [name, callback, attributes, options],
  [],
  [],
  []
);
