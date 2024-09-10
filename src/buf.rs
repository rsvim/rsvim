//! The VIM buffer.

#![allow(dead_code)]

use parking_lot::RwLock;
use ropey::{Rope, RopeBuilder};
use std::collections::BTreeMap;
use std::convert::From;
use std::sync::{Arc, Weak};
use std::time::Duration;

use crate::{glovar, uuid};

pub type BufferId = usize;

#[derive(Clone, Debug)]
/// The VIM buffer.
pub struct Buffer {
  id: BufferId,
  rope: Rope,
}

pub type BufferArc = Arc<RwLock<Buffer>>;
pub type BufferWk = Weak<RwLock<Buffer>>;

impl Buffer {
  pub fn new() -> Self {
    Buffer {
      id: uuid::next(),
      rope: Rope::new(),
    }
  }

  pub fn to_arc(b: Buffer) -> BufferArc {
    Arc::new(RwLock::new(b))
  }

  pub fn id(&self) -> BufferId {
    self.id
  }

  pub fn rope(&self) -> &Rope {
    &self.rope
  }

  pub fn rope_mut(&mut self) -> &mut Rope {
    &mut self.rope
  }
}

impl Default for Buffer {
  fn default() -> Self {
    Buffer::new()
  }
}

impl From<Rope> for Buffer {
  fn from(rope: Rope) -> Self {
    Buffer {
      id: uuid::next(),
      rope,
    }
  }
}

impl From<RopeBuilder> for Buffer {
  fn from(builder: RopeBuilder) -> Self {
    Buffer {
      id: uuid::next(),
      rope: builder.finish(),
    }
  }
}

impl PartialEq for Buffer {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for Buffer {}

#[derive(Debug, Clone)]
/// The manager for all buffers.
pub struct Buffers {
  // Buffers collection
  buffers: BTreeMap<BufferId, BufferArc>,
}

impl Buffers {
  pub fn new() -> Self {
    Buffers {
      buffers: BTreeMap::new(),
    }
  }

  pub fn to_arc(b: Buffers) -> BuffersArc {
    Arc::new(RwLock::new(b))
  }

  pub fn is_empty(&self) -> bool {
    self.buffers.is_empty()
  }

  pub fn len(&self) -> usize {
    self.buffers.len()
  }

  pub fn insert(&mut self, buffer: BufferArc) -> Option<BufferArc> {
    let buffer_id = buffer
      .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap()
      .id();
    self.buffers.insert(buffer_id, buffer)
  }

  pub fn remove(&mut self, id: &BufferId) -> Option<BufferArc> {
    self.buffers.remove(id)
  }

  pub fn get(&self, id: &BufferId) -> Option<&BufferArc> {
    self.buffers.get(id)
  }

  pub fn contains_key(&self, id: &BufferId) -> bool {
    self.buffers.contains_key(id)
  }

  pub fn keys(&self) -> std::collections::btree_map::Keys<BufferId, BufferArc> {
    self.buffers.keys()
  }

  pub fn values(&self) -> std::collections::btree_map::Values<BufferId, BufferArc> {
    self.buffers.values()
  }

  pub fn iter(&self) -> std::collections::btree_map::Iter<BufferId, BufferArc> {
    self.buffers.iter()
  }

  pub fn first_key_value(&self) -> Option<(&BufferId, &BufferArc)> {
    self.buffers.first_key_value()
  }

  pub fn last_key_value(&self) -> Option<(&BufferId, &BufferArc)> {
    self.buffers.last_key_value()
  }
}

impl Default for Buffers {
  fn default() -> Self {
    Buffers::new()
  }
}

pub type BuffersArc = Arc<RwLock<Buffers>>;
pub type BuffersWk = Weak<RwLock<Buffers>>;

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use tempfile::tempfile;

  #[test]
  fn buffer_from1() {
    let rop1 = Rope::from_str("Hello");
    let buf1: Buffer = rop1.into();
    let tmp1 = tempfile().unwrap();
    buf1.rope.write_to(tmp1).unwrap();

    let rop2 = Rope::from_reader(File::open("Cargo.toml").unwrap()).unwrap();
    let buf2 = Buffer::from(rop2);
    let tmp2 = tempfile().unwrap();
    buf2.rope.write_to(tmp2).unwrap();
  }

  #[test]
  fn buffer_from2() {
    let mut builder1 = RopeBuilder::new();
    builder1.append("Hello");
    builder1.append("World");
    let buf1: Buffer = builder1.into();
    let tmp1 = tempfile().unwrap();
    buf1.rope.write_to(tmp1).unwrap();
  }
}
