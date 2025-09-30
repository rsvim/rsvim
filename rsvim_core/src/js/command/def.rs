//! Ex command definition.

use crate::js::command::attr::*;
use crate::js::command::opt::*;
use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use std::fmt::Debug;
use std::rc::Rc;

pub type CommandCallback = Rc<v8::Global<v8::Function>>;

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
      .field("name", &self.name)
      .field("callback", &"Rc<v8::Global<v8::Function>>")
      .field("attributes", &self.attributes)
      .field("options", &self.options)
      .finish()
  }
}

impl CommandDefinition {
  pub fn from_v8_object(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
  ) -> Self {
    debug_assert!(args.length() == 4);
    let name = args.get(0).to_rust_string_lossy(scope);
    let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let callback = Rc::new(v8::Global::new(scope, callback));
    let attributes = args.get(2).to_object(scope).unwrap();
    let attributes = CommandAttributes::from_v8_object(scope, attributes);
    let options = args.get(3).to_object(scope).unwrap();
    let options = CommandOptions::from_v8_object(scope, options);

    Self {
      name: name.to_compact_string(),
      callback,
      attributes,
      options,
    }
  }

  pub fn into_v8_object<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    // name
    let name_field = v8::String::new(scope, "name").unwrap();
    let name_value = v8::String::new(scope, &self.name).unwrap();
    obj.set(scope, name_field.into(), name_value.into());

    // callback
    let callback_field = v8::String::new(scope, "callback").unwrap();
    let callback_value = v8::Local::new(scope, (*self.callback).clone());
    obj.set(scope, callback_field.into(), callback_value.into());

    // attributes
    let attr_field = v8::String::new(scope, "attributes").unwrap();
    let attr_value = self.attributes.into_v8_object(scope);
    obj.set(scope, attr_field.into(), attr_value.into());

    // options
    let opts_field = v8::String::new(scope, "options").unwrap();
    let opts_value = self.options.into_v8_object(scope);
    obj.set(scope, opts_field.into(), opts_value.into());

    obj
  }
}
