//! Ex command definition.

use crate::js::binding;
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
    let name = String::from_v8(scope, args.get(0));
    let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
    let callback = CallbackFromV8::from_v8(scope, callback);
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

to_v8_impl!(CommandDefinition, [name, callback]);

impl ToV8 for CommandDefinition {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // name
    let name_value = to_v8(scope, self.name.clone());
    binding::set_property_to(scope, obj, NAME, name_value);

    // callback
    let callback_value = v8::Local::new(scope, (*self.callback).clone());
    binding::set_property_to(scope, obj, CALLBACK, callback_value.into());

    // attributes
    let attr_value = to_v8(scope, self.attributes.clone());
    binding::set_property_to(scope, obj, ATTRIBUTES, attr_value);

    // options
    let opts_value = to_v8(scope, self.options.clone());
    binding::set_property_to(scope, obj, OPTIONS, opts_value);

    obj.into()
  }
}
