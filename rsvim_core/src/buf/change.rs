//! Buffer changes.

// use crate::buf::text::Text;
// use crate::prelude::*;
use compact_str::CompactString;
// use path_absolutize::Absolutize;
// use std::fs::Metadata;
// use std::path::Path;
// use std::path::PathBuf;
use tokio::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert {
  char_idx: usize,
  payload: CompactString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delete {
  char_idx: usize,
  n: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Basic unit of a change operation:
/// - Insert payload at an absolute char index.
/// - Delete `n` characters at an absolute char index.
///
/// The "Replace" operation can be converted into delete+insert operations.
///
/// The change operation doesn't maintain current cursor's position, so a
/// buffer can change without need to know where the cursor is.
///
/// NOTE: Ropey provide two types of coordinate system:
/// 1. 2-Dimension on line number and char index per line.
/// 2. 1-Dimension on absolute char index per whole buffer.
pub enum Operation {
  Insert(Insert),
  Delete(Delete),
}

#[derive(Debug, Clone)]
pub struct Changes {
  ops: Vec<Operation>,
  timestamp: Instant,
}

impl Changes {
  pub fn operations(&self) -> &Vec<Operation> {
    &self.ops
  }

  pub fn timestamp(&self) -> &Instant {
    &self.timestamp
  }

  pub fn update_timestamp(&mut self) {
    self.timestamp = Instant::now();
  }

  pub fn delete(&mut self, char_idx: usize, n: usize) {
    if n == 0 {
      return;
    }

    if let Some(Operation::Delete(delete)) = self.ops.last_mut()
      && delete.char_idx == char_idx
    {
      // Merge two deletion
      delete.n += n;
    } else if let Some(Operation::Delete(delete)) = self.ops.last_mut()
      && delete.char_idx > char_idx
      && delete.char_idx - char_idx <= n
    {
      // Merge two deletion
      delete.char_idx = char_idx;
      delete.n += n;
    } else {
      self.ops.push(Operation::Delete(Delete { char_idx, n }));
    }
  }

  pub fn insert(&mut self, char_idx: usize, payload: CompactString) {
    if payload.is_empty() {
      return;
    }

    if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && (char_idx >= insert.char_idx
        || char_idx < insert.char_idx + insert.payload.len())
    {
      // Merge two insertion
      insert
        .payload
        .insert_str(char_idx - insert.char_idx, &payload);
    } else if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && (char_idx == insert.char_idx + insert.payload.len())
    {
      // Merge two insertion
      insert.payload.push_str(&payload);
    } else {
      self
        .ops
        .push(Operation::Insert(Insert { char_idx, payload }));
    }
  }
}
