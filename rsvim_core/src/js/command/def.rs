//! Ex command definition.

use crate::js::command::attr::*;
use crate::js::command::opt::*;
use std::rc::Rc;

pub type CommandCallback = Rc<v8::Global<v8::Function>>;

#[derive(Debug, Clone)]
pub struct CommandDefinition {
  pub callback: CommandCallback,
  pub attributes: CommandAttributes,
  pub options: CommandOptions,
}

impl CommandDefinition {
  pub fn from_v8_object<'a>(
    scope: &mut v8::HandleScope,
    args: v8::FunctionCallbackArguments,
  ) -> Self {
    debug_assert!(args.length() == 4);
    // First argument is command "name".
    let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let callback = Rc::new(v8::Global::new(scope, callback));
    let attributes = args.get(2).to_object(scope).unwrap();
    let attributes = CommandAttributes::from_v8_object(scope, attributes);
    let options = args.get(3).to_object(scope).unwrap();
    let options = CommandOptions::from_v8_object(scope, options);

    Self {
      callback,
      attributes,
      options,
    }
  }
}
