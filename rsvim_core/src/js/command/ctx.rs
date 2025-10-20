//! Ex command runtime context.

use crate::js::binding;
use crate::js::converter::*;
use crate::to_v8;
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
    let bang_value = to_v8!(bool scope, self.bang);
    binding::set_property_to(scope, obj, BANG, bang_value);

    // args
    if let Some(args) = self.args.clone() {
      let args_value = to_v8!(bool scope, args);
      binding::set_property_to(scope, obj, ARGS, args_value);
    }

    obj.into()
  }
}
