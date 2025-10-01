//! Ex command definition.

use crate::js::command::attr::*;
use crate::js::command::opt::*;
use crate::js::converter::*;
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

impl FromV8CallbackArguments for CommandDefinition {
  fn from_v8_callback_arguments(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
  ) -> Option<Self> {
    debug_assert!(args.length() == 4);
    let name = args.get(0).to_rust_string_lossy(scope);
    let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let callback = Rc::new(v8::Global::new(scope, callback));
    let attributes = CommandAttributes::from_v8(scope, args.get(2)).unwrap();
    let options = CommandOptions::from_v8(scope, args.get(3)).unwrap();

    Some(Self {
      name: name.to_compact_string(),
      callback,
      attributes,
      options,
    })
  }
}

impl ToV8 for CommandDefinition {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> Option<v8::Local<'s, v8::Value>> {
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
    let attr_value = self.attributes.to_v8(scope);
    obj.set(scope, attr_field.into(), attr_value.into());

    // options
    let opts_field = v8::String::new(scope, "options").unwrap();
    let opts_value = self.options.to_v8(scope);
    obj.set(scope, opts_field.into(), opts_value.into());

    obj.into()
  }
}
