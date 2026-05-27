//! Child-process command options.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

/// Default command options.
pub const CWD_DEFAULT: Option<CompactString> = None;
pub const CLEAR_ENV_DEFAULT: bool = false;
pub const STDIN_DEFAULT: &str = "null";

#[derive(
  Debug,
  Clone,
  PartialEq,
  Eq,
  derive_builder::Builder,
  rsvim_macro::ToV8,
  rsvim_macro::FromV8,
)]
pub struct ProcCommandOptions {
  #[builder(default = Vec::new())]
  pub args: Vec<CompactString>,

  #[builder(default = CWD_DEFAULT)]
  pub cwd: Option<CompactString>,

  #[builder(default = CLEAR_ENV_DEFAULT)]
  pub clear_env: bool,

  #[builder(default = FoldMap::new())]
  pub envs: FoldMap<CompactString, CompactString>,

  #[builder(default = STDIN_DEFAULT.to_compact_string())]
  pub stdin: CompactString,
}
