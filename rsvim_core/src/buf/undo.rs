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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert {
  pub payload: CompactString,
  pub timestamp: Instant,
  pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delete {
  pub payload: CompactString,
  pub timestamp: Instant,
  pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retain {
  pub char_idx: usize,
  pub timestamp: Instant,
  pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Basic unit of a change operation:
/// - Insert
/// - Delete
/// - Retain: Cursor moves to an absolute char position
///
/// The "Replace" operation can be converted into delete+insert operations.
///
/// NOTE: The `char_idx` in operation is absolute char index in the buffer
/// text.
pub enum Operation {
  Insert(Insert),
  Delete(Delete),
  Retain(Retain),
}

#[derive(Debug, Default, Clone)]
pub struct Changes {
  ops: Vec<Operation>,
}

impl Changes {
  pub fn new() -> Self {
    Self { ops: vec![] }
  }

  pub fn operations(&self) -> &Vec<Operation> {
    &self.ops
  }

  pub fn operations_mut(&mut self) -> &mut Vec<Operation> {
    &mut self.ops
  }

  pub fn retain(&mut self, char_idx: usize, version: usize) {
    self.ops.push(Operation::Retain(Retain {
      char_idx,
      timestamp: Instant::now(),
      version,
    }));
  }

  pub fn delete(&mut self, payload: CompactString, version: usize) {
    if payload.is_empty() {
      return;
    }

    if let Some(Operation::Delete(last)) = self.ops.last_mut() {
      // Merge two deletion
      trace!("self.ops.last-1, last:{:?}, payload:{:?}", last, payload);
      last.payload.push_str(&payload);
    } else if let Some(Operation::Insert(last)) = self.ops.last_mut()
      && last.payload == payload
    {
      // Remove last insertion
      trace!("self.ops.last-2, last:{:?}, payload:{:?}", last, payload);
      self.ops.pop();
    } else {
      self.ops.push(Operation::Delete(Delete {
        payload,
        timestamp: Instant::now(),
        version,
      }));
    }
  }

  pub fn insert(
    &mut self,
    char_idx: usize,
    payload: CompactString,
    version: usize,
  ) {
    let payload_chars_count = payload.chars().count();
    let last_payload_chars_count = self.ops.last().map(|l| match l {
      Operation::Insert(insert) => insert.payload.chars().count(),
      Operation::Delete(delete) => delete.payload.chars().count(),
    });

    if payload_chars_count == 0 {
      return;
    }

    if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && char_idx >= insert.char_idx
      && char_idx < insert.char_idx + last_payload_chars_count.unwrap()
    {
      trace!(
        "self.ops.last-1, char_idx:{:?},payload.count:{:?}",
        insert.char_idx, last_payload_chars_count
      );
      // Merge two insertion
      insert
        .payload
        .insert_str(char_idx - insert.char_idx, &payload);
    } else if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && char_idx == insert.char_idx + last_payload_chars_count.unwrap()
    {
      trace!(
        "self.ops.last-2, char_idx:{:?},payload.count:{:?}",
        insert.char_idx, last_payload_chars_count
      );
      // Merge two insertion
      insert.payload.push_str(&payload);
    } else {
      self.ops.push(Operation::Insert(Insert {
        char_idx,
        payload,
        timestamp: Instant::now(),
        version,
      }));
    }
  }
}

pub struct UndoManager {
  history: LocalRb<Heap<Operation>>,
  changes: Changes,
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
      .field("changes", &self.changes)
      .field("__next_version", &self.__next_version)
      .finish()
  }
}

impl UndoManager {
  pub fn new() -> Self {
    Self {
      history: LocalRb::new(100),
      changes: Changes::new(),
      __next_version: 0,
    }
  }

  fn next_version(&mut self) -> usize {
    self.__next_version += 1;
    self.__next_version
  }

  pub fn changes(&self) -> &Changes {
    &self.changes
  }

  pub fn insert(&mut self, char_idx: usize, payload: CompactString) {
    let version = self.next_version();
    self.changes.insert(char_idx, payload, version);
  }

  pub fn delete(&mut self, char_idx: usize, payload: CompactString) {
    let version = self.next_version();
    self.changes.delete(char_idx, payload, version);
  }

  pub fn commit(&mut self) {
    for change in self.changes.operations_mut().drain(..) {
      self.history.push_overwrite(change);
    }
    self.changes = Changes::new();
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
