//! Syntax query.

use crate::prelude::*;
use std::fmt::Debug;
use std::sync::Arc;
use tree_sitter::Query;

pub type SyntaxQueryArc = Arc<Query>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Line (row) index and column (byte) index.
pub struct SyntaxQueryCaptureKey {
  row: usize,
  column: usize,
}

impl SyntaxQueryCaptureKey {
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SyntaxQueryCaptureValue {
  index: u32,
  range: tree_sitter::Range,
}

impl SyntaxQueryCaptureValue {
  pub fn new(index: u32, range: tree_sitter::Range) -> Self {
    Self { index, range }
  }

  pub fn index(&self) -> u32 {
    self.index
  }

  pub fn range(&self) -> &tree_sitter::Range {
    &self.range
  }
}

pub type SyntaxQueryCaptureMap =
  FoldMap<SyntaxQueryCaptureKey, Vec<SyntaxQueryCaptureValue>>;

#[derive(Debug)]
pub struct SyntaxQueryCapture {
  // Maps start_point to all its captured nodes.
  nodes: SyntaxQueryCaptureMap,
}

arc_ptr!(SyntaxQueryCapture);

impl SyntaxQueryCapture {
  pub fn new(nodes: SyntaxQueryCaptureMap) -> Self {
    Self { nodes }
  }

  pub fn nodes(&self) -> &SyntaxQueryCaptureMap {
    &self.nodes
  }
}
