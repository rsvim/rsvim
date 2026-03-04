//! Syntax query.

use crate::prelude::*;
use compact_str::CompactString;
use std::fmt::Debug;
use std::sync::Arc;
use tree_sitter::Query;

pub type SyntaxQueryArc = Arc<Query>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Line (row) index and column (byte) index.
pub struct SyntaxCaptureKey {
  row: usize,
  column: usize,
}

impl SyntaxCaptureKey {
  pub fn new(row: usize, column: usize) -> Self {
    Self { row, column }
  }

  pub fn row(&self) -> usize {
    self.row
  }

  pub fn column(&self) -> usize {
    self.column
  }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SyntaxCaptureValue {
  name: CompactString,
  range: tree_sitter::Range,
}

impl SyntaxCaptureValue {
  pub fn new(name: CompactString, range: tree_sitter::Range) -> Self {
    Self { name, range }
  }

  pub fn name(&self) -> &CompactString {
    &self.name
  }

  pub fn range(&self) -> &tree_sitter::Range {
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
