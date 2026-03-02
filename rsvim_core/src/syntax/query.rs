//! Syntax query.

use crate::prelude::*;
use std::fmt::Debug;
use std::sync::Arc;
use tree_sitter::Query;

pub type SynQueryArc = Arc<Query>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
/// Line (row) index and column (byte) index.
pub struct SynCaptureKey {
  row: usize,
  column: usize,
}

impl SynCaptureKey {
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
pub struct SynCaptureValue {
  index: u32,
  range: tree_sitter::Range,
}

impl SynCaptureValue {
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

pub type SynCaptureMap = FoldMap<SynCaptureKey, Vec<SynCaptureValue>>;

#[derive(Debug)]
pub struct SynCapture {
  // Maps start_point to all its captured nodes.
  nodes: SynCaptureMap,
}

arc_ptr!(SynCapture);

impl SynCapture {
  pub fn new(nodes: SynCaptureMap) -> Self {
    Self { nodes }
  }

  pub fn nodes(&self) -> &SynCaptureMap {
    &self.nodes
  }
}
