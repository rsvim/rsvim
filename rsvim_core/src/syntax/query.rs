//! Syntax query.

use crate::prelude::*;
use compact_str::CompactString;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::sync::Arc;
use tree_sitter::Query;

pub type SyntaxQueryArc = Arc<Query>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SyntaxCapturePoint {
  pub line_idx: usize,
  pub char_idx: usize,
}

impl Ord for SyntaxCapturePoint {
  fn cmp(&self, other: &Self) -> Ordering {
    if self.line_idx != other.line_idx {
      self.line_idx.cmp(&other.line_idx)
    } else {
      self.char_idx.cmp(&other.char_idx)
    }
  }
}

impl PartialOrd for SyntaxCapturePoint {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Convert [`tree_sitter::Range`] based bytes indexing into [`ropey::Rope`]
/// based chars indexing, i.e. use [`ropey::Rope::byte_to_char`] API to
/// transform byte index to char index.
pub struct SyntaxCaptureRange {
  pub start_char: usize,
  pub end_char: usize,
  pub start_point: SyntaxCapturePoint,
  pub end_point: SyntaxCapturePoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxCaptureValue {
  pub index: u32,
  pub name: CompactString,
  pub range: SyntaxCaptureRange,
}

#[derive(Debug, Clone)]
pub struct SyntaxCaptureMultipleValues {
  pub values: Vec<SyntaxCaptureValue>,
  pub max_end_char: Option<usize>,
  pub max_end_point: Option<SyntaxCapturePoint>,
}

pub type SyntaxCaptureMap =
  FoldMap<SyntaxCapturePoint, SyntaxCaptureMultipleValues>;

#[derive(Debug)]
pub struct SyntaxCapture {
  // Maps start_point to all its captured nodes.
  nodes: SyntaxCaptureMap,
}

arc_ptr!(SyntaxCapture);

impl SyntaxCapture {
  pub fn new(nodes: SyntaxCaptureMap) -> Self {
    Self { nodes }
  }

  pub fn nodes(&self) -> &SyntaxCaptureMap {
    &self.nodes
  }
}
