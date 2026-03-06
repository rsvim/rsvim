//! Syntax query.

use crate::prelude::*;
use compact_str::CompactString;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::sync::Arc;
use tree_sitter::Query;

pub type SyntaxQueryArc = Arc<Query>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Line (row) index and column (byte) index.
pub struct SyntaxCaptureKey {
  line_idx: usize,
  char_idx: usize,
}

impl SyntaxCaptureKey {
  pub fn new(line_idx: usize, char_idx: usize) -> Self {
    Self { line_idx, char_idx }
  }

  pub fn line_idx(&self) -> usize {
    self.line_idx
  }

  pub fn char_idx(&self) -> usize {
    self.char_idx
  }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SyntaxCapturePoint {
  pub line_idx: usize,
  pub char_idx: usize,
}

impl Ord for SyntaxCapturePoint {
  fn cmp(&self, other: &Self) -> Ordering {
    if self.line_idx < other.line_idx {
      Ordering::Less
    } else if self.line_idx > other.line_idx {
      Ordering::Greater
    } else {
      if self.char_idx < other.char_idx {
        Ordering::Less
      } else if self.char_idx > other.char_idx {
        Ordering::Greater
      } else {
        Ordering::Equal
      }
    }
  }
}

impl PartialOrd for SyntaxCapturePoint {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    if self.line_idx < other.line_idx {
      Some(Ordering::Less)
    } else if self.line_idx > other.line_idx {
      Some(Ordering::Greater)
    } else {
      if self.char_idx < other.char_idx {
        Some(Ordering::Less)
      } else if self.char_idx > other.char_idx {
        Some(Ordering::Greater)
      } else {
        Some(Ordering::Equal)
      }
    }
  }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Convert [`tree_sitter::Range`] based bytes indexing into [`ropey::Rope`]
/// based chars indexing, i.e. use [`ropey::Rope::byte_to_char`] API to
/// transform byte index to char index.
pub struct SyntaxCaptureRange {
  pub start_char: usize,
  pub end_char: usize,
  pub start_point: SyntaxCapturePoint,
  pub end_point: SyntaxCapturePoint,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SyntaxCaptureValue {
  index: u32,
  name: CompactString,
  range: SyntaxCaptureRange,
  max_end_char: usize,
  max_end_point: SyntaxCapturePoint,
}

impl SyntaxCaptureValue {
  pub fn new(
    index: u32,
    name: CompactString,
    range: SyntaxCaptureRange,
    max_end_char: usize,
    max_end_point: SyntaxCapturePoint,
  ) -> Self {
    Self { index, name, range }
  }

  pub fn index(&self) -> u32 {
    self.index
  }

  pub fn name(&self) -> &CompactString {
    &self.name
  }

  pub fn range(&self) -> &SyntaxCaptureRange {
    &self.range
  }
}

pub type SyntaxCaptureMap = FoldMap<SyntaxCaptureKey, Vec<SyntaxCaptureValue>>;

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
