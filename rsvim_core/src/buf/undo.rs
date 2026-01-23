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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeleteDirection {
  ToLeft,
  ToRight,
}

pub trait FindDeleteDirection {
  fn direction(&self) -> DeleteDirection;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert {
  /// Absolute char idx on insertion.
  pub char_idx: usize,
  pub payload: CompactString,

  /// Cursor's absolute char idx before doing insertion.
  pub cursor_char_idx_before: usize,
  /// Cursor's absolute char idx after doing insertion.
  pub cursor_char_idx_after: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delete {
  /// Absolute char idx on deletion.
  pub char_idx: usize,
  pub payload: CompactString,

  /// Cursor's absolute char idx before doing insertion.
  pub cursor_char_idx_before: usize,
  /// Cursor's absolute char idx after doing insertion.
  pub cursor_char_idx_after: usize,
}

impl FindDeleteDirection for Delete {
  fn direction(&self) -> DeleteDirection {
    debug_assert_ne!(self.cursor_char_idx_before, self.cursor_char_idx_after);
    if self.cursor_char_idx_after > self.cursor_char_idx_before {
      DeleteDirection::ToRight
    } else {
      DeleteDirection::ToLeft
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A change is either a [`Insert`] or a [`Delete`].
/// The "Replace" operation can be converted into delete+insert operations.
pub enum Change {
  Insert(Insert),
  Delete(Delete),
}

#[derive(Debug, Default, Clone)]
/// A commit is a basic unit of undo/redo. It can contains one or more changes.
///
/// Multiple insertions/deletions can be merged into one change. For some use
/// cases, this can reduce the changes length inside one commit:
///
/// 1. Insert continuously chars `Hello, World`, actually we create 12
///    insertions: `H`, `e`, `l`, `l`, `o`, `,`, ` `, `W`, `o`, `r`, `l`, `d`.
///    We can merge these insertions into 1 change `Hello, World`.
/// 2. First insert a char `a`, then delete it. Or first delete a char `b`,
///    then insert it back. Such kind of changes can be deduplicated.
pub struct Commit {
  changes: Vec<Change>,
}

impl Commit {
  pub fn new() -> Self {
    Self { changes: vec![] }
  }

  pub fn operations(&self) -> &Vec<Change> {
    &self.changes
  }

  pub fn operations_mut(&mut self) -> &mut Vec<Change> {
    &mut self.changes
  }

  pub fn delete(&mut self, op: Delete) {
    debug_assert!(
      op.cursor_char_idx_after + op.payload.chars().count()
        == op.cursor_char_idx_before
        || op.cursor_char_idx_before + op.payload.chars().count()
          == op.cursor_char_idx_after
    );

    if op.payload.is_empty() {
      return;
    }

    if let Some(Change::Delete(last)) = self.changes.last_mut() {
      // Merge two deletion
      trace!("self.ops.last-1, last:{:?}, payload:{:?}", last, payload);
      last.payload.push_str(&payload);
    } else if let Some(Change::Insert(last)) = self.changes.last_mut()
      && last.payload == payload
    {
      // Remove last insertion
      trace!("self.ops.last-2, last:{:?}, payload:{:?}", last, payload);
      self.changes.pop();
    } else {
      self.changes.push(Change::Delete(Delete {
        payload,
        timestamp: Instant::now(),
        version,
      }));
    }
  }

  pub fn insert(&mut self, op: Insert) {
    debug_assert_eq!(
      op.cursor_char_idx_before + op.payload.chars().count(),
      op.cursor_char_idx_after
    );

    if op.payload.is_empty() {
      return;
    }

    let payload_chars_count = payload.chars().count();
    let last_payload_chars_count = self.changes.last().map(|l| match l {
      Change::Insert(insert) => insert.payload.chars().count(),
      Change::Delete(delete) => delete.payload.chars().count(),
    });

    if payload_chars_count == 0 {
      return;
    }

    if let Some(Change::Insert(insert)) = self.changes.last_mut()
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
    } else if let Some(Change::Insert(insert)) = self.changes.last_mut()
      && char_idx == insert.char_idx + last_payload_chars_count.unwrap()
    {
      trace!(
        "self.ops.last-2, char_idx:{:?},payload.count:{:?}",
        insert.char_idx, last_payload_chars_count
      );
      // Merge two insertion
      insert.payload.push_str(&payload);
    } else {
      self.changes.push(Change::Insert(Insert {
        char_idx,
        payload,
        timestamp: Instant::now(),
        version,
      }));
    }
  }
}

pub struct UndoManager {
  history: LocalRb<Heap<Change>>,
  changes: Commit,
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
      changes: Commit::new(),
      __next_version: START_VERSION,
    }
  }

  fn next_version(&mut self) -> usize {
    let result = self.__next_version;
    self.__next_version += 1;
    result
  }

  pub fn changes(&self) -> &Commit {
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
    self.changes = Commit::new();
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
