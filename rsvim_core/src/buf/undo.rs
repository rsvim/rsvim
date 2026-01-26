//! Undo history.

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

pub const INVALID_VERSION: usize = 0;
pub const START_VERSION: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert {
  pub payload: CompactString,

  /// Cursor's absolute char idx before doing insertion.
  /// This is also the absolute insertion char index.
  pub char_idx_before: usize,

  /// Cursor's absolute char idx after doing insertion.
  pub char_idx_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delete {
  pub payload: CompactString,

  /// Cursor's absolute char idx before doing deletion.
  /// This is also the absolute deletion char index.
  pub char_idx_before: usize,

  /// Cursor's absolute char idx after doing deletion.
  pub char_idx_after: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeleteDirection {
  ToLeft,
  ToRight,
}

impl Delete {
  fn direction(&self) -> DeleteDirection {
    debug_assert!(self.char_idx_after <= self.char_idx_before);
    if self.char_idx_after < self.char_idx_before {
      DeleteDirection::ToLeft
    } else {
      DeleteDirection::ToRight
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// An operation is either a [`Insert`] or a [`Delete`].
/// The "Replace" operation can be converted into "Delete"+"Insert" operations.
///
/// Multiple operations can be merged into one operation. This can reduce
/// unnecessary operations inside one commit:
///
/// 1. Insert continuously chars `Hello, World`, actually we create 12
///    insertions: `H`, `e`, `l`, `l`, `o`, `,`, ` `, `W`, `o`, `r`, `l`, `d`.
///    We can merge these insertions into one `Hello, World`.
/// 2. First insert a char `a`, then delete it. Or first delete a char `b`,
///    then insert it back. Such kind of deletions can be deduplicated.
pub enum Operation {
  Insert(Insert),
  Delete(Delete),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A record for operation with timestamp.
pub struct Record {
  pub op: Operation,
  pub timestamp: Instant,
  pub version: usize,
}

#[derive(Debug, Default, Clone)]
/// Undo manager maintains two parts:
/// 1. Uncommitted changes: When user starts insert mode, we will create a new
///    `Commit` struct to store all the uncommitted changes the user is going
///    to do.
/// 2. Committed history: When user finishes typing and switches to
///    other modes (normal, visual, etc) from insert mode, we will commit the
///    uncommitted changes to committed history.
///
/// NOTE:
/// 1. A operation record is a basic unit of undo/redo operation. Each time
///    user performs a undo/redo, the user operates on a operation record.
/// 2. For committed history, operation records will not be merged.
/// 3. For uncommitted changes, even they can be merged, there still can have
///    more than 1 operations. When we commit them, we will commit all of them
///    to undo manager.
pub struct Current {
  records: Vec<Record>,
}

impl Current {
  pub fn new() -> Self {
    Self { records: vec![] }
  }

  pub fn records(&self) -> &Vec<Record> {
    &self.records
  }

  pub fn records_mut(&mut self) -> &mut Vec<Record> {
    &mut self.records
  }

  pub fn delete(&mut self, op: Delete) {
    debug_assert!(
      op.char_idx_after == op.char_idx_before
        || op.char_idx_before == op.char_idx_after + op.payload.chars().count()
    );

    if op.payload.is_empty() {
      return;
    }

    if let Some(last_record) = self.records.last_mut()
      && let Operation::Delete(ref mut last) = last_record.op
      && last.direction() == DeleteDirection::ToLeft
      && op.direction() == DeleteDirection::ToLeft
      && op.char_idx_before == last.char_idx_after
    {
      // Merge 2 deletions to left
      trace!("last-1:{:?}, op:{:?}", last, op);
      last.payload.insert_str(0, &op.payload);
      last.char_idx_after = op.char_idx_after;
      last_record.timestamp = Instant::now();
    } else if let Some(last_record) = self.records.last_mut()
      && let Operation::Delete(ref mut last) = last_record.op
      && last.direction() == DeleteDirection::ToRight
      && op.direction() == DeleteDirection::ToRight
      && op.char_idx_before == last.char_idx_after
    {
      // Merge 2 deletions to right
      trace!("last-2:{:?}, op:{:?}", last, op);
      last.payload.push_str(&op.payload);
      last.char_idx_after = op.char_idx_after;
      last_record.timestamp = Instant::now();
    } else if let Some(last_record) = self.records.last_mut()
      && let Operation::Insert(ref mut last) = last_record.op
      && last.payload == op.payload
      && ((last.char_idx_before == op.char_idx_after
        && last.char_idx_after == op.char_idx_before
        && op.direction() == DeleteDirection::ToLeft)
        || (last.char_idx_before == op.char_idx_before
          && last.char_idx_after == op.char_idx_after
          && op.direction() == DeleteDirection::ToRight))
    {
      // Offset the effect of 1 insertion and 1 deletion
      trace!("last-3:{:?}, op:{:?}", last, op);
      self.records.pop();
    } else {
      trace!("last-4, op:{:?}", op);
      self.records.push(Record {
        op: Operation::Delete(op),
        timestamp: Instant::now(),
        version: INVALID_VERSION,
      });
    }
  }

  pub fn insert(&mut self, op: Insert) {
    debug_assert_eq!(
      op.char_idx_before + op.payload.chars().count(),
      op.char_idx_after
    );

    if op.payload.is_empty() {
      return;
    }

    if let Some(last_record) = self.records.last_mut()
      && let Operation::Insert(ref mut last) = last_record.op
      && last.char_idx_after == op.char_idx_before
    {
      trace!("last-1:{:?}, op:{:?}", last, op);
      // Append to last insertion
      last.payload.push_str(&op.payload);
      last.char_idx_after = op.char_idx_after;
      last_record.timestamp = Instant::now();
    } else {
      trace!("last-2, op:{:?}", op);
      self.records.push(Record {
        op: Operation::Insert(op),
        timestamp: Instant::now(),
        version: INVALID_VERSION,
      });
    }
  }
}

pub struct UndoManager {
  history: LocalRb<Heap<Record>>,
  current: Current,
  __next_version: usize,
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
      .field("changes", &self.current)
      .field("__next_version", &self.__next_version)
      .finish()
  }
}

impl UndoManager {
  pub fn new() -> Self {
    Self {
      history: LocalRb::new(100),
      current: Current::new(),
      __next_version: START_VERSION,
    }
  }

  fn next_version(&mut self) -> usize {
    let result = self.__next_version;
    self.__next_version += 1;
    result
  }

  pub fn current(&self) -> &Current {
    &self.current
  }

  pub fn insert(&mut self, op: Insert) {
    self.current.insert(op);
  }

  pub fn delete(&mut self, op: Delete) {
    self.current.delete(op);
  }

  pub fn commit(&mut self) {
    let version = self.next_version();
    for mut change in self.current.records_mut().drain(..) {
      change.version = version;
      self.history.push_overwrite(change);
    }
    self.current = Current::new();
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
