//! Editing/change history, useful for undo/redo.

use crate::buf::BufferId;
use crate::buf::text::Text;
use crate::prelude::*;
use compact_str::CompactString;
use ringbuf::LocalRb;
use ringbuf::storage::Heap;
use ringbuf::traits::Observer;
use ringbuf::traits::RingBuffer;
use std::fmt::Debug;
use tokio::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert {
  pub char_idx: usize,
  pub payload: CompactString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delete {
  pub char_idx: usize,
  pub payload: CompactString,
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

  pub fn save(&mut self, op: Operation) {
    match op {
      Operation::Insert(insert) => self.insert(insert.char_idx, insert.payload),
      Operation::Delete(delete) => self.delete(delete.char_idx, delete.payload),
    }
  }

  pub fn delete(&mut self, char_idx: usize, payload: CompactString) {
    if n == 0 {
      return;
    }

    if let Some(Operation::Delete(delete)) = self.ops.last_mut()
      && delete.char_idx == char_idx
    {
      // Merge two deletion
      trace!(
        "self.ops.last-1, char_idx:{:?},n:{:?}",
        delete.char_idx, delete.n
      );
      delete.n += n;
    } else if let Some(Operation::Delete(delete)) = self.ops.last_mut()
      && delete.char_idx > char_idx
      && delete.char_idx - char_idx <= n
    {
      trace!(
        "self.ops.last-2, char_idx:{:?},n:{:?}",
        delete.char_idx, delete.n
      );
      // Merge two deletion
      delete.char_idx = char_idx;
      delete.n += n;
    } else if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && insert.char_idx == char_idx
      && insert.payload.chars().count() == n
    {
      trace!(
        "self.ops.last-3, char_idx:{:?},payload.count:{:?}",
        insert.char_idx,
        insert.payload.chars().count()
      );
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
      trace!(
        "self.ops.last-1, char_idx:{:?},payload.count:{:?}",
        insert.char_idx,
        insert.payload.chars().count()
      );
      // Merge two insertion
      insert
        .payload
        .insert_str(char_idx - insert.char_idx, &payload);
    } else if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && (char_idx == insert.char_idx + insert.payload.chars().count())
    {
      trace!(
        "self.ops.last-2, char_idx:{:?},payload.count:{:?}",
        insert.char_idx,
        insert.payload.chars().count()
      );
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

pub struct UndoManager {
  history: LocalRb<Heap<Change>>,
  current: Change,
  next_version: usize,
}

impl Default for UndoManager {
  fn default() -> Self {
    Self::new()
  }
}

impl Debug for UndoManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("UndoManager")
      .field("history_occupied_len", &self.history.occupied_len())
      .field("history_vacant_len", &self.history.vacant_len())
      .field("current", &self.current)
      .field("next_version", &self.next_version)
      .finish()
  }
}

impl UndoManager {
  pub fn new() -> Self {
    let version = 1;
    Self {
      history: LocalRb::new(100),
      current: Change::new(version),
      next_version: version + 1,
    }
  }

  pub fn current(&self) -> &Change {
    &self.current
  }

  pub fn save(&mut self, op: Operation) {
    self.current.save(op);
  }

  pub fn commit(&mut self) {
    self.history.push_overwrite(self.current.clone());
    self.current = Change::new(self.next_version);
    self.next_version += 1;
  }

  /// This is similar to `git revert` a specific git commit ID.
  ///
  /// It reverts to the previous `commit`.
  ///
  /// Returns `Ok` and modifies the passed `text` if revert successfully,
  /// returns `Err` and not change the `text` if `I` is not exist in history.
  pub fn revert(
    &mut self,
    commit: usize,
    buf_id: BufferId,
    _text: &mut Text,
  ) -> TheResult<()> {
    if commit >= self.history.occupied_len() {
      return Err(TheErr::UndoCommitNotExist(commit, buf_id));
    }

    Ok(())
  }

  pub fn max_commit(&self) -> usize {
    self.history.occupied_len()
  }
}
