//! Vim buffers.

use crate::buf::opt::BufferLocalOptions;
use crate::defaults::grapheme::AsciiControlCode;

use compact_str::CompactString;
use parking_lot::RwLock;
use ropey::iter::Lines;
use ropey::{Rope, RopeBuilder, RopeSlice};
use std::collections::BTreeMap;
use std::convert::From;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};
use unicode_width::UnicodeWidthChar;

pub mod opt;

/// Buffer ID.
pub type BufferId = i32;

/// Next unique buffer ID.
///
/// NOTE: Start form 1.
pub fn next_buffer_id() -> BufferId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Debug)]
/// The Vim buffer.
pub struct Buffer {
  id: BufferId,
  rope: Rope,
  options: BufferLocalOptions,
}

pub type BufferArc = Arc<RwLock<Buffer>>;
pub type BufferWk = Weak<RwLock<Buffer>>;

impl Buffer {
  /// Make buffer with default [`BufferLocalOptions`].
  pub fn new() -> Self {
    Buffer {
      id: next_buffer_id(),
      rope: Rope::new(),
      options: BufferLocalOptions::default(),
    }
  }

  pub fn to_arc(b: Buffer) -> BufferArc {
    Arc::new(RwLock::new(b))
  }

  pub fn id(&self) -> BufferId {
    self.id
  }
}

// Unicode {
impl Buffer {
  /// Get the display width for a unicode `char`.
  pub fn char_width(&self, c: char) -> usize {
    if c.is_ascii_control() {
      let cc = AsciiControlCode::try_from(c).unwrap();
      match cc {
        AsciiControlCode::Ht => self.tab_stop() as usize,
        AsciiControlCode::Lf => 0,
        _ => format!("{}", cc).len(),
      }
    } else {
      UnicodeWidthChar::width_cjk(c).unwrap()
    }
  }

  /// Get the printable cell symbol and its display width.
  pub fn char_symbol(&self, c: char) -> (CompactString, usize) {
    let width = self.char_width(c);
    if c.is_ascii_control() {
      let cc = AsciiControlCode::try_from(c).unwrap();
      match cc {
        AsciiControlCode::Ht => (
          CompactString::from(" ".repeat(self.tab_stop() as usize)),
          width,
        ),
        AsciiControlCode::Lf => (CompactString::new(""), width),
        _ => (CompactString::from(format!("{}", cc)), width),
      }
    } else {
      (CompactString::from(c.to_string()), width)
    }
  }
}
// Unicode }

// Rope {
impl Buffer {
  pub fn get_line(&self, line_idx: usize) -> Option<RopeSlice> {
    self.rope.get_line(line_idx)
  }

  pub fn get_lines_at(&self, line_idx: usize) -> Option<Lines> {
    self.rope.get_lines_at(line_idx)
  }

  pub fn lines(&self) -> Lines {
    self.rope.lines()
  }

  pub fn write_to<T: std::io::Write>(&self, writer: T) -> std::io::Result<()> {
    self.rope.write_to(writer)
  }

  pub fn append(&mut self, other: Rope) -> &mut Self {
    self.rope.append(other);
    self
  }
}
// Rope }

impl Default for Buffer {
  fn default() -> Self {
    Buffer::new()
  }
}

// Options {
impl Buffer {
  pub fn options(&self) -> &BufferLocalOptions {
    &self.options
  }

  pub fn set_options(&mut self, options: &BufferLocalOptions) {
    self.options = options.clone();
  }

  pub fn tab_stop(&self) -> u16 {
    self.options.tab_stop()
  }

  pub fn set_tab_stop(&mut self, value: u16) {
    self.options.set_tab_stop(value);
  }
}
// Options }

impl From<Rope> for Buffer {
  /// Make buffer from [`Rope`].
  fn from(rope: Rope) -> Self {
    Buffer {
      id: next_buffer_id(),
      rope,
      options: BufferLocalOptions::default(),
    }
  }
}

impl From<RopeBuilder> for Buffer {
  /// Make buffer from [`RopeBuilder`].
  fn from(builder: RopeBuilder) -> Self {
    Buffer {
      id: next_buffer_id(),
      rope: builder.finish(),
      options: BufferLocalOptions::default(),
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

  // Local options for buffers.
  local_options: BufferLocalOptions,
}

impl Buffers {
  pub fn new() -> Self {
    Buffers {
      buffers: BTreeMap::new(),
      local_options: BufferLocalOptions::default(),
    }
  }

  pub fn to_arc(b: Buffers) -> BuffersArc {
    Arc::new(RwLock::new(b))
  }

  pub fn new_buffer(&mut self) -> BufferId {
    let mut buf = Buffer::new();
    buf.set_options(self.local_options());
    let buf_id = buf.id();
    self.buffers.insert(buf_id, Buffer::to_arc(buf));
    buf_id
  }

  pub fn new_buffer_from_rope(&mut self, rope: Rope) -> BufferId {
    let mut buf = Buffer::from(rope);
    buf.set_options(self.local_options());
    let buf_id = buf.id();
    self.buffers.insert(buf_id, Buffer::to_arc(buf));
    buf_id
  }

  pub fn new_buffer_from_rope_builder(&mut self, rope_builder: RopeBuilder) -> BufferId {
    let mut buf = Buffer::from(rope_builder);
    buf.set_options(self.local_options());
    let buf_id = buf.id();
    self.buffers.insert(buf_id, Buffer::to_arc(buf));
    buf_id
  }
}

// BTreeMap {
impl Buffers {
  pub fn is_empty(&self) -> bool {
    self.buffers.is_empty()
  }

  pub fn len(&self) -> usize {
    self.buffers.len()
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

  pub fn keys(&self) -> BuffersKeys {
    self.buffers.keys()
  }

  pub fn values(&self) -> BuffersValues {
    self.buffers.values()
  }

  pub fn iter(&self) -> BuffersIter {
    self.buffers.iter()
  }

  pub fn first_key_value(&self) -> Option<(&BufferId, &BufferArc)> {
    self.buffers.first_key_value()
  }

  pub fn last_key_value(&self) -> Option<(&BufferId, &BufferArc)> {
    self.buffers.last_key_value()
  }
}
// BTreeMap }

impl Default for Buffers {
  fn default() -> Self {
    Buffers::new()
  }
}

// Options {
impl Buffers {
  pub fn local_options(&self) -> &BufferLocalOptions {
    &self.local_options
  }

  pub fn set_local_options(&mut self, options: &BufferLocalOptions) {
    self.local_options = options.clone();
  }
}
// Options }

pub type BuffersArc = Arc<RwLock<Buffers>>;
pub type BuffersWk = Weak<RwLock<Buffers>>;
pub type BuffersKeys<'a> = std::collections::btree_map::Keys<'a, BufferId, BufferArc>;
pub type BuffersValues<'a> = std::collections::btree_map::Values<'a, BufferId, BufferArc>;
pub type BuffersIter<'a> = std::collections::btree_map::Iter<'a, BufferId, BufferArc>;

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use tempfile::tempfile;

  #[test]
  fn buffer_from1() {
    let r1 = Rope::from_str("Hello");
    let buf1 = Buffer::from(r1);
    let tmp1 = tempfile().unwrap();
    buf1.write_to(tmp1).unwrap();

    let r2 = Rope::from_reader(File::open("Cargo.toml").unwrap()).unwrap();
    let buf2 = Buffer::from(r2);
    let tmp2 = tempfile().unwrap();
    buf2.write_to(tmp2).unwrap();
  }

  #[test]
  fn buffer_from2() {
    let mut builder1 = RopeBuilder::new();
    builder1.append("Hello");
    builder1.append("World");
    let buf1 = Buffer::from(builder1);
    let tmp1 = tempfile().unwrap();
    buf1.write_to(tmp1).unwrap();
  }

  #[test]
  fn next_buffer_id1() {
    assert!(next_buffer_id() > 0);
  }
}
