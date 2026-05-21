//! Ex command options.

use crate::js::converter::*;
use compact_str::CompactString;

/// Command option names.
pub const FORCE: &str = "force";
pub const ALIAS: &str = "alias";

/// Default command options.
pub const FORCE_DEFAULT: bool = true;
pub const ALIAS_DEFAULT: Option<CompactString> = None;

#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  derive_builder::Builder,
  rsvim_macro::ToV8,
  rsvim_macro::FromV8,
)]
pub struct CommandOptions {
  #[builder(default = FORCE_DEFAULT)]
  #[from_v8_bool]
  pub force: bool,

  #[builder(default = ALIAS_DEFAULT)]
  #[from_v8_string]
  pub alias: Option<CompactString>,
}

// impl FromV8 for CommandOptions {
//   fn from_v8<'s>(
//     scope: &mut v8::PinScope<'s, '_>,
//     obj: v8::Local<'s, v8::Value>,
//   ) -> Self {
//     debug_assert!(obj.is_object() || obj.is_object_template());
//     let obj = obj.to_object(scope).unwrap();
//     let mut builder = CommandOptionsBuilder::default();
//
//     from_v8_prop!(builder, obj, scope, bool, force);
//     from_v8_prop!(builder, obj, scope, CompactString, alias, optional);
//
//     builder.build().unwrap()
//   }
// }
