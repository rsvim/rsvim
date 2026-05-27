//! Ex command options.

use compact_str::CompactString;

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
  pub force: bool,

  #[builder(default = ALIAS_DEFAULT)]
  pub alias: Option<CompactString>,
}
