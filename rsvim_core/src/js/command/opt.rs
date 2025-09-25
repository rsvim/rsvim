//! Ex command options.

pub const FORCE: &str = "force";

#[derive(Debug, Clone)]
pub struct CommandOptions {
  pub force: bool,
}
