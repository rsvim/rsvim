//! Ex command runtime context.

use crate::js::converter::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

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
  bang: bool,

  #[builder(default = ARGS_DEFAULT)]
  args: Vec<CompactString>,
}

impl CommandContext {
  pub fn bang(&self) -> bool {
    self.bang
  }

  pub fn args(&self) -> &Vec<CompactString> {
    &self.args
  }
}

impl FromV8 for CommandContext {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    let mut builder = CommandContextBuilder::default();
    let obj = value.to_object(scope).unwrap();

    // bang
    let bang_name = to_v8(scope, BANG);
    if let Some(bang_value) = obj.get(scope, bang_name) {
      builder.bang(from_v8::<bool>(scope, bang_value));
    }

    // nargs
    let nargs_name = to_v8(scope, NARGS);
    if let Some(nargs_value) = obj.get(scope, nargs_name) {
      let nargs = from_v8::<CompactString>(scope, nargs_value);
      builder.nargs(Nargs::from_str(&nargs).unwrap());
    }

    builder.build().unwrap()
  }
}

impl ToV8 for CommandContext {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // bang
    let bang_field = to_v8(scope, BANG);
    let bang_value = to_v8(scope, self.bang());
    obj.set(scope, bang_field, bang_value);

    // nargs
    let nargs_field = to_v8(scope, NARGS);
    let nargs_value = to_v8(scope, self.nargs.to_compact_string());
    obj.set(scope, nargs_field, nargs_value);

    obj.into()
  }
}
