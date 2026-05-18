//! Ex command options.

use crate::from_v8_prop;
use crate::js::converter::*;
use compact_str::CompactString;
use rsvim_macro::ToV8;

/// Command option names.
pub const FORCE: &str = "force";
pub const ALIAS: &str = "alias";

/// Default command options.
pub const FORCE_DEFAULT: bool = true;
pub const ALIAS_DEFAULT: Option<CompactString> = None;

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder, ToV8)]
pub struct CommandOptions {
  #[builder(default = FORCE_DEFAULT)]
  pub force: bool,

  #[builder(default = ALIAS_DEFAULT)]
  pub alias: Option<CompactString>,
}

impl StructFromV8 for CommandOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = CommandOptionsBuilder::default();

    from_v8_prop!(builder, obj, scope, bool, force);
    from_v8_prop!(builder, obj, scope, CompactString, alias, optional);

    builder.build().unwrap()
  }
}
