//! Ex command runtime context.

use crate::js::converter::*;
use compact_str::CompactString;

/// Command attribute name.
pub const BANG: &str = "bang";
pub const ARGS: &str = "args";

/// Default command attributes.
pub const BANG_DEFAULT: bool = false;
pub const ARGS_DEFAULT: Vec<CompactString> = vec![];

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct CommandContext {
  #[builder(default = BANG_DEFAULT)]
  // bang
  pub bang: bool,

  #[builder(default = ARGS_DEFAULT)]
  pub args: Vec<CompactString>,
}

impl ToV8 for CommandContext {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // bang
    let bang_field = v8::String::new(scope, BANG).unwrap();
    let bang_value = to_v8(scope, self.bang);
    obj.define_own_property(
      scope,
      bang_field.into(),
      bang_value,
      v8::PropertyAttribute::READ_ONLY,
    );

    // args
    let args_field = v8::String::new(scope, ARGS).unwrap();
    let args_value = to_v8(scope, self.args.clone());
    obj.define_own_property(
      scope,
      args_field.into(),
      args_value,
      v8::PropertyAttribute::READ_ONLY,
    );

    obj.into()
  }
}
