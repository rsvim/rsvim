//! Ex command options.

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
  #[from_v8(bool)]
  pub force: bool,

  #[builder(default = ALIAS_DEFAULT)]
  #[from_v8(string)]
  pub alias: Option<CompactString>,
}
