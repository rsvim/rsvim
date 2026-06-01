//! Child-process command options.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;

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
  #[builder(default = vec![])]
  pub args: Vec<CompactString>,

  #[builder(default = None)]
  pub cwd: Option<CompactString>,

  #[builder(default = false)]
  pub clear_env: bool,

  #[builder(default = FoldMap::new())]
  pub env: FoldMap<CompactString, CompactString>,

  #[builder(default = "null".to_compact_string())]
  pub stdin: CompactString,
}
