//! Editing/change history, useful for undo/redo.

use crate::prelude::*;
use std::fmt::Debug;
// use crate::buf::text::Text;
// use crate::prelude::*;
use compact_str::CompactString;
// use path_absolutize::Absolutize;
// use std::fs::Metadata;
// use std::path::Path;
// use std::path::PathBuf;
use ringbuf::HeapRb;
use ringbuf::traits::Observer;
use ringbuf::traits::RingBuffer;
use tokio::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert {
  pub char_idx: usize,
  pub payload: CompactString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delete {
  pub char_idx: usize,
  pub n: usize,
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
pub struct Change {
  ops: Vec<Operation>,
  timestamp: Instant,
  version: usize,
}

impl Change {
  pub fn new(version: usize) -> Self {
    Self {
      ops: vec![],
      timestamp: Instant::now(),
      version,
    }
  }

  pub fn operations(&self) -> &Vec<Operation> {
    &self.ops
  }

  pub fn timestamp(&self) -> &Instant {
    &self.timestamp
  }

  pub fn version(&self) -> usize {
    self.version
  }

  fn update_timestamp(&mut self) {
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
      trace!("self.ops.last-1, char_idx:{:?},n:{:?}", delete.char_idx, delete.n);
      delete.n += n;
    } else if let Some(Operation::Delete(delete)) = self.ops.last_mut()
      && delete.char_idx > char_idx
      && delete.char_idx - char_idx <= n
    {
      trace!("self.ops.last-2, char_idx:{:?},n:{:?}", delete.char_idx, delete.n);
      // Merge two deletion
      delete.char_idx = char_idx;
      delete.n += n;
    } else if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && insert.char_idx == char_idx
      && insert.payload.chars().count() == n
    {
      trace!("self.ops.last-3, char_idx:{:?},payload.count:{:?}", insert.char_idx, insert.payload.chars().count());
      // Cancel both insertion and deletion
      self.ops.pop();
    } else {
      self.ops.push(Operation::Delete(Delete { char_idx, n }));
    }

    self.update_timestamp();
  }

  pub fn insert(&mut self, char_idx: usize, payload: CompactString) {
    if payload.is_empty() {
      return;
    }

    if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && char_idx >= insert.char_idx
      && char_idx < insert.char_idx + insert.payload.chars().count()
    {
      trace!("self.ops.last-1, char_idx:{:?},payload.count:{:?}", insert.char_idx, insert.payload.chars().count());
      // Merge two insertion
      insert
        .payload
        .insert_str(char_idx - insert.char_idx, &payload);
    } else if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && (char_idx == insert.char_idx + insert.payload.chars().count())
    {
      trace!("self.ops.last-2, char_idx:{:?},payload.count:{:?}", insert.char_idx, insert.payload.chars().count());
      // Merge two insertion
      insert.payload.push_str(&payload);
    } else {
      self
        .ops
        .push(Operation::Insert(Insert { char_idx, payload }));
    }

    self.update_timestamp();
  }
}

pub struct ChangeManager {
  change_history: HeapRb<Change>,
  current_change: Change,
  next_version: usize,
}

impl Default for ChangeManager {
  fn default() -> Self {
    Self::new()
  }
}

impl Debug for ChangeManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ChangeManager")
      .field(
        "change_history_occupied_len",
        &self.change_history.occupied_len(),
      )
      .field(
        "change_history_vacant_len",
        &self.change_history.vacant_len(),
      )
      .field("current_change", &self.current_change)
      .field("next_version", &self.next_version)
      .finish()
  }
}

impl ChangeManager {
  pub fn new() -> Self {
    let version = 1;
    Self {
      change_history: HeapRb::new(100),
      current_change: Change::new(version),
      next_version: version + 1,
    }
  }

  pub fn current_change(&self) -> &Change {
    &self.current_change
  }

  pub fn save(&mut self, op: Operation) {
    match op {
      Operation::Delete(delete) => {
        self.current_change.delete(delete.char_idx, delete.n)
      }
      Operation::Insert(insert) => {
        self.current_change.insert(insert.char_idx, insert.payload)
      }
    }
  }

  pub fn commit(&mut self) {
    self
      .change_history
      .push_overwrite(self.current_change.clone());
    self.current_change = Change::new(self.next_version);
    self.next_version += 1;
  }
}
