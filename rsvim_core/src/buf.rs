//! Vim buffers.

use crate::buf::opt::BufferLocalOptions;
use crate::defaults::grapheme::AsciiControlCode;

use compact_str::CompactString;
use parking_lot::RwLock;
use ropey::iter::Lines;
use ropey::{Rope, RopeBuilder, RopeSlice};
use std::collections::BTreeMap;
use std::convert::From;
use std::marker::PhantomData;
use std::ptr::NonNull;
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

#[derive(Clone, Debug, Default)]
/// The prefix-sum like tree structure.
///
/// It maps from the last char index and its display width, and the opposite direction (from the
/// display width to the last char index). And it is only a cache that when
/// [`Viewport`](crate::ui::widget::window::viewport::Viewport) renders a buffer, especially not
/// from the first char of the line.
///
/// When rendering without a cache, it will have to go through all the chars (to sum up all the
/// display width) before the real first displayed char/column in the line, thus ensure the correct
/// display column for the first char.
///
/// In most cases, cursor moves in successive positions, i.e. cursor moves left/right/up/down,
/// while completely randomly move is a rare use case.
///
/// And editing a line is a common use case, i.e. chars can be added/removed and changed, and the
/// display width change as well. Most third party prefix-sum implementations are fixed length,
/// which leads to a performance issue. If user keeps adding new chars at the end of the line,
/// third party prefix-sum will use `O(N)` to add a single char (where the `N` is the length of the
/// line), which can be slow.
///
/// On the contrary, this struct always uses `O(M * log_N)` (where the `M` is the distance between
/// current start display column and previous one, the `N` is the length of the line):
///
/// 1. Add/remove chars successively in a line (`M` is 1).
/// 2. Cursor moves in successive positions (`M` is 1).
/// 3. Cursor moves to a random position (`M` is the length of the movement).
/// 4. Bulk delete chars in a line (`M` is the chars count of the bulk delete).
///
/// For example:
///
/// ```text
/// ^@^A^B^C^D^E^F^G^H<--HT-->
/// ^K^L^M^N^O^P^Q^R^S^T^U^V^W^X^Y^Z^[^\^]^^^_
/// 你好，Vim！
/// こんにちは、Vim！
/// 안녕 Vim!
/// ```
///
/// The above example shows that a unicode character could uses more than 1 cells width to display
/// in the terminal:
///
/// 1. For the 1~2 lines are ASCII control codes (0~31), the tab (`HT`, renders as `<--HT-->`) uses
///    8 empty cells by default, the new line (`LF`) uses no cells but simply starts another new
///    line.
/// 2. For the other lines, they are Chinese/Japanese/Korean characters, use 2 cells width to
///    display in terminal.
///
struct PrefixWidth {
  // Maps from char index to display width.
  char2width: BTreeMap<usize, usize>,
  // Maps from display width to the last char index.
  width2char: BTreeMap<usize, usize>,
}

impl PrefixWidth {
  /// Get the display width by the specific char.
  ///
  /// Returns
  ///
  /// 1. `None` if the char not exist in the buffer line.
  /// 2. Display width if the char exists in the buffer line.
  pub fn get_width(&self, char_idx: usize) -> Option<usize> {
    match self.char2width.get(&char_idx) {
      Some(width) => Some(*width),
      None => None,
    }
  }

  /// Get the (last) char index by the display width.
  ///
  /// Returns
  ///
  /// 1. `None` if the buffer line is shroter than the width.
  /// 2. Last char index if the width exists in the buffer line.
  pub fn get_char(&self, width: usize) -> Option<usize> {
    match self.width2char.get(&width) {
      Some(char_idx) => Some(*char_idx),
      None => None,
    }
  }

  /// Set the display width on a specific char.
  pub fn set_width(&mut self, char_idx: usize, width: usize) {
    self.char2width.insert(char_idx, width);
    self.width2char.insert(width, char_idx);
  }

  /// Clear data from the specific char.
  ///
  /// This is usually because the char is been modified/removed from the line.
  pub fn clear_by_char(&mut self, start_char_idx: usize) {
    if start_char_idx == 0 {
      self.char2width.clear();
      self.width2char.clear();
    } else {
      self
        .char2width
        .retain(|&char_idx, _| char_idx < start_char_idx);
      match self.char2width.last_key_value() {
        Some((_, &last_width)) => {
          self.width2char.retain(|&width, _| width <= last_width);
        }
        None => {
          self.width2char.clear();
        }
      }
    }
  }

  /// Clear data from the specific width.
  pub fn clear_by_width(&mut self, start_width: usize) {}
}

type LinesIndex = BTreeMap<usize, PrefixWidth>;

#[derive(Clone, Debug)]
/// The Vim buffer.
pub struct Buffer {
  id: BufferId,
  rope: Rope,
  lines_index: LinesIndex,
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
      lines_index: BTreeMap::new(),
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

pub struct BufferColumnIndexMut<'s> {
  buffer: NonNull<Buffer>,
  buffer_phantom: PhantomData<&'s mut Buffer>,
  column_index: NonNull<PrefixWidth>,
  column_index_phantom: PhantomData<&'s mut PrefixWidth>,
  line_slice: RopeSlice<'s>,
}

impl<'s> BufferColumnIndexMut<'s> {
  pub fn new(
    buffer: NonNull<Buffer>,
    column_index: NonNull<PrefixWidth>,
    line_slice: RopeSlice<'s>,
  ) -> Self {
    Self {
      buffer,
      buffer_phantom: PhantomData,
      column_index,
      column_index_phantom: PhantomData,
      line_slice,
    }
  }

  /// Get the line width (on the specific line index) from the first char index (i.e. 0) until the
  /// specific char index (i.e. the last char index).
  ///
  /// Returns
  ///
  /// The display width (based on cells on the terminal) from first char index 0 to last char
  /// index.
  pub fn get_width_until_char(&mut self, char_idx: usize) -> Option<usize> {
    unsafe {
      let mut column_index = self.column_index;
      let line_slice = self.line_slice;
      let buffer = self.buffer;

      // If found the cached result for the line.
      match column_index.as_ref().get_width(char_idx) {
        Some(width) => {
          // Found cached result, directly return it.
          return Some(width);
        }
        None => { /* No cached value in column index */ }
      }

      // If not found the cached result, we need to go through the line and calculate the result,
      // also cache the results for future query.
      let mut line_width = 0_usize;
      for (i, c) in line_slice.chars().enumerate() {
        let c_width = match column_index.as_ref().get_width(i) {
          Some(c_width) => {
            // If the char index `i` is already cached.
            c_width
          }
          None => {
            // If the char index `i` is not cached.
            let c_width = buffer.as_ref().char_width(c);
            for w in (line_width + 1)..(line_width + c_width + 1) {
              column_index.as_mut().set_width(i, w);
            }
            c_width
          }
        };
        line_width += c_width;
      }

      Some(line_width)
    }
  }

  /// Get the last char index (on the specific line index) indicate by the line width.
  ///
  /// NOTE: The last char need to be completely accommodated in this line. An edge case is: if the
  /// last char takes more than 1 cells width on the terminal, and the line width is not exactly on
  /// the end of the char. For example:
  ///
  /// ```text
  ///  0                                33       34
  ///  |                                |        |
  /// |--------------------------------------|
  /// |This is the beginning of the very <--H|T--> long line.
  /// |--------------------------------------|
  ///  |                                    |
  ///  0                                    37
  /// ```
  ///
  /// The example shows for the line, when display width is 37, the last char index that is
  /// completely accommodated inside it is 33. The 34 (tab, `<--HT-->`) uses 8 spaces and the right
  /// part of it goes out of the line width.
  ///
  /// Returns
  ///
  /// 1. The display width (based on cells on the terminal) from first char index 0 to last char
  ///    index.
  /// 2. `None` if the line or char index doesn't exist in the buffer.
  pub fn get_last_char_by_width(&mut self, width: usize) -> Option<usize> {
    unsafe {
      let mut column_index = self.column_index;
      let line_slice = self.line_slice;
      let buffer = self.buffer;

      // If found the cached result for the width.
      match column_index.as_ref().get_char(width) {
        Some(char_idx) => {
          // Found cached result, directly return it.
          return Some(char_idx);
        }
        None => { /* No cached value in column index */ }
      }

      // We need to go through the line and calculate the result, also cache the results for future
      // query.
      let mut line_width = 0_usize;
      for (i, c) in line_slice.chars().enumerate() {
        let c_width = match column_index.as_ref().get_width(i) {
          Some(c_width) => {
            // If the char index `i` is already cached.
            c_width
          }
          None => {
            // If the char index `i` is not cached.
            let c_width = buffer.as_ref().char_width(c);
            for w in (line_width + 1)..(line_width + c_width + 1) {
              column_index.as_mut().set_width(i, w);
            }
            c_width
          }
        };

        if line_width + c_width > width {
          return Some(i);
        }

        line_width += c_width;
      }

      // Cannot find `width` in current line.
      None
    }
  }
}

// Column index {
impl Buffer {
  /// Get column index on a specific line.
  ///
  /// This is a special index that helps query each character and its display width on the line of
  /// the buffer, since a unicode char can vary the display width in terminal cells.
  ///
  /// Returns
  ///
  /// 1. The column index iterator on the line.
  /// 2. `None` if the line doesn't exist in the buffer.
  pub fn get_column_index(&mut self, line_idx: usize) -> Option<BufferColumnIndexMut> {
    unsafe {
      let mut raw_self = NonNull::new(self as *mut Buffer).unwrap();

      let line_slice = match raw_self.as_ref().rope.get_line(line_idx) {
        Some(line_slice) => line_slice,
        None => {
          // The line doesn't exist in rope, directly returns none.
          return None;
        }
      };

      let mut raw_lines_index =
        NonNull::new(&mut raw_self.as_mut().lines_index as *mut LinesIndex).unwrap();

      match raw_lines_index.as_mut().get_mut(&line_idx) {
        Some(_) => { /* Nothing */ }
        None => {
          // Initialize column index if not exist
          raw_lines_index
            .as_mut()
            .insert(line_idx, PrefixWidth::default());
        }
      }

      let column_index = raw_lines_index.as_mut().get_mut(&line_idx).unwrap();
      let column_index = NonNull::new(column_index as *mut PrefixWidth).unwrap();
      Some(BufferColumnIndexMut::new(
        raw_self,
        column_index,
        line_slice,
      ))
    }
  }

  /// Clear column indexes from specific line.
  ///
  /// Once the buffer is been truncated/modified on some lines, this method should be use to reset
  /// the caches on column indexes.
  pub fn clear_column_index(&mut self, start_line_idx: usize) {
    self
      .lines_index
      .retain(|&line_idx, _| line_idx < start_line_idx);
  }
}
// Column index }

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
    // Append operation can affected the last 1~2 lines index.
    let last_line_idx = self.rope.len_lines();
    let start_line_idx = if last_line_idx > 2 {
      last_line_idx - 2
    } else {
      0
    };
    self.clear_column_index(start_line_idx);
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
      lines_index: BTreeMap::new(),
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
      lines_index: BTreeMap::new(),
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
