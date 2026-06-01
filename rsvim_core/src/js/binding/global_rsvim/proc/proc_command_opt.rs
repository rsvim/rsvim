//! Child-process command options.

use crate::js::converter::*;
use crate::prelude::*;
use compact_str::CompactString;
use std::str::FromStr;

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum Stdio {
  #[strum(serialize = "null")]
  Null,

  #[strum(serialize = "piped")]
  Piped,

  #[strum(serialize = "inherit")]
  Inherit,
}

impl FromV8 for Stdio {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    debug_assert!(value.is_string() || value.is_string_object());
    let result = value.to_string(scope).unwrap().to_rust_string_lossy(scope);
    Stdio::from_str(&result).unwrap()
  }
}

impl ToV8 for Stdio {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    self.to_string().to_v8(scope)
  }
}

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

  #[builder(default = Stdio::Null)]
  pub stdin: Stdio,

  #[builder(default = Stdio::Piped)]
  pub stdout: Stdio,

  #[builder(default = Stdio::Piped)]
  pub stderr: Stdio,
}
