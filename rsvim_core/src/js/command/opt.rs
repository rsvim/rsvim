//! Ex command options.

/// Command option names.
pub const FORCE_NAME: &str = "force";

/// Default command options.
pub const FORCE_VALUE: bool = true;

#[derive(Debug, Clone)]
pub struct CommandOptions {
  pub force: bool,
}
