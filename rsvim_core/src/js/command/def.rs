//! Ex command definition.

use crate::js::command::attr::*;
use crate::js::command::opt::*;
use std::fmt::Debug;
use std::rc::Rc;

pub type CommandCallback = Rc<v8::Global<v8::Function>>;

#[derive(Clone)]
pub struct CommandDefinition {
  pub name: String,
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

  pub fn into_v8_object<'a>(
    &self,
    scope: &mut v8::HandleScope<'a>,
    name: &str,
  ) -> v8::Local<'a, v8::Object> {
    let cmd = v8::Object::new(scope);

    // name
    let name_field = v8::String::new(scope, "name").unwrap();
    let name_value = v8::String::new(scope, name.as_ref()).unwrap();
    cmd.set(scope, name_field.into(), name_value.into());

    // attributes
    let attr_field = v8::String::new(scope, "attributes").unwrap();
    let attr_value = def.attributes.into_v8_object(scope);
    cmd.set(scope, attr_field.into(), attr_value.into());

    // options
    let opts_field = v8::String::new(scope, "options").unwrap();
    let opts_value = def.options.into_v8_object(scope);
    cmd.set(scope, opts_field.into(), opts_value.into());

    cmd
  }
}

impl Debug for CommandDefinition {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CommandDefinition")
      .field("attributes", &self.attributes)
      .field("options", &self.options)
      .field("callback", &"v8::Global<v8::Function>")
      .finish()
  }
}
