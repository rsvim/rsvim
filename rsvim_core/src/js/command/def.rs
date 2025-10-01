//! Ex command definition.

use crate::js::command::attr::*;
use crate::js::command::opt::*;
use crate::js::converter::*;
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

impl FromV8CallbackArguments for CommandDefinition {
  fn from_v8_callback_arguments<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    args: v8::FunctionCallbackArguments<'s>,
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
    let name_field = to_v8(scope, &NAME).unwrap();
    let name_value = to_v8(scope, &self.name).unwrap();
    obj.set(scope, name_field.into(), name_value.into());

    // callback
    let callback_field = to_v8(scope, &CALLBACK).unwrap();
    let callback_value = v8::Local::new(scope, (*self.callback).clone());
    obj.set(scope, callback_field.into(), callback_value.into());

    // attributes
    let attr_field = to_v8(scope, &ATTRIBUTES).unwrap();
    let attr_value = to_v8(scope, &self.attributes).unwrap();
    obj.set(scope, attr_field, attr_value);

    // options
    let opts_field = to_v8(scope, &OPTIONS).unwrap();
    let opts_value = to_v8(scope, &self.options).unwrap();
    obj.set(scope, opts_field.into(), opts_value);

    Some(obj.into())
  }
}
