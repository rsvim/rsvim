//! Syntax editings.

use ropey::Rope;
use std::fmt::Debug;
use tree_sitter::InputEdit;

#[derive(Clone)]
pub struct SynEditNew {
  pub payload: Rope,
  pub version: isize,
}

impl Debug for SynEditNew {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxEditNew")
      .field(
        "payload",
        &self
          .payload
          .get_line(0)
          .map(|l| l.to_string())
          .unwrap_or("".to_string()),
      )
      .field("version", &self.version)
      .finish()
  }
}

#[derive(Clone)]
pub struct SynEditUpdate {
  pub payload: Rope,
  pub input: InputEdit,
  pub version: isize,
}

impl Debug for SynEditUpdate {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SyntaxEditUpdate")
      .field(
        "payload",
        &self
          .payload
          .get_line(0)
          .map(|l| l.to_string())
          .unwrap_or("".to_string()),
      )
      .field("input", &self.input)
      .field("version", &self.version)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub enum SynEdit {
  New(SynEditNew),
  Update(SynEditUpdate),
}
