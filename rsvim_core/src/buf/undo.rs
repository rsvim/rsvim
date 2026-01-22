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
struct Insert2 {
  pub char_idx: usize,
  pub payload: CompactString,
  pub timestamp: Instant,
  pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Delete2 {
  pub char_idx: usize,
  pub payload: CompactString,
  pub timestamp: Instant,
  pub version: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Basic unit of a change operation:
/// - Insert
/// - Delete
///
/// The "Replace" operation can be converted into delete+insert operations.
///
/// Operations don't maintain the cursor's position, so a buffer can change
/// without the need to know where the cursor is.
///
/// NOTE: The `char_idx` in operation is absolute char index in the buffer
/// text.
enum Operation {
  Insert(Insert2),
  Delete(Delete2),
}

#[derive(Debug, Clone)]
pub struct Change {
  ops: Vec<Operation>,
}

impl Change {
  pub fn new() -> Self {
    Self { ops: vec![] }
  }

  pub fn operations(&self) -> &Vec<Operation> {
    &self.ops
  }

  pub fn save(&mut self, op: Operation, version: usize) {
    match op {
      Operation::Insert(insert) => {
        self.insert(insert.char_idx, insert.payload, version)
      }
      Operation::Delete(delete) => {
        self.delete(delete.char_idx, delete.payload, version)
      }
    }
  }

  fn delete(
    &mut self,
    char_idx: usize,
    payload: CompactString,
    version: usize,
  ) {
    let payload_chars_count = payload.chars().count();
    if payload_chars_count == 0 {
      return;
    }

    if let Some(Operation::Delete(delete)) = self.ops.last_mut()
      && delete.char_idx == char_idx
    {
      // Merge two deletion
      trace!(
        "self.ops.last-1, char_idx:{:?}, payload:{:?}",
        char_idx, payload
      );
      delete.payload.push_str(&payload);
    } else if let Some(Operation::Delete(delete)) = self.ops.last_mut()
      && delete.char_idx > char_idx
      && delete.char_idx - char_idx <= payload_chars_count
    {
      trace!(
        "self.ops.last-2, char_idx:{:?}, payload:{:?}",
        char_idx, payload
      );
      let first = payload
        .chars()
        .take(delete.char_idx - char_idx)
        .collect::<CompactString>();
      let second = payload
        .chars()
        .skip(delete.char_idx - char_idx)
        .collect::<CompactString>();
      // Merge two deletion
      delete.char_idx = char_idx;
      if first.chars().count() > 0 {
        delete.payload.insert_str(0, &first);
      }
      if second.chars().count() > 0 {
        delete.payload.push_str(&second);
      }
    } else if let Some(Operation::Insert(insert)) = self.ops.last_mut()
      && insert.char_idx == char_idx
      && insert.payload == payload
    {
      trace!(
        "self.ops.last-3, char_idx:{:?}, payload:{:?}",
        char_idx, payload
      );
      // Cancel both insertion and deletion
      self.ops.pop();
    } else {
      self
        .ops
        .push(Operation::Delete(Delete2 { char_idx, payload }));
    }
  }

  fn insert(
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
      self
        .ops
        .push(Operation::Insert(Insert2 { char_idx, payload }));
    }

    self.update_timestamp();
  }
}

pub struct UndoManager {
  history: LocalRb<Heap<Change>>,
  current: Change,
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
      .field("current", &self.current)
      .field("__next_version", &self.__next_version)
      .finish()
  }
}

impl UndoManager {
  pub fn new() -> Self {
    Self {
      history: LocalRb::new(100),
      current: Change::new(),
      __next_version: 0,
    }
  }

  fn next_version(&mut self) -> usize {
    self.__next_version += 1;
    self.__next_version
  }

  pub fn current(&self) -> &Change {
    &self.current
  }

  pub fn insert(&mut self, char_idx: usize, payload: CompactString) {
    let version = self.next_version();
    self.current.insert(char_idx, payload, version);
  }

  pub fn delete(&mut self, char_idx: usize, payload: CompactString) {
    let version = self.next_version();
    self.current.delete(char_idx, payload, version);
  }

  pub fn commit(&mut self) {
    self.history.push_overwrite(self.current.clone());
    self.current = Change::new();
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
