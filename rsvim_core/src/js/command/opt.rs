//! Ex command options.

/// Command option names.
pub const FORCE_NAME: &str = "force";

/// Default command options.
pub const FORCE_VALUE: bool = true;

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
pub struct CommandOptions {
  #[builder(default = FORCE_VALUE)]
  pub force: bool,
}
