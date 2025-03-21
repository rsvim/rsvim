//! Vim buffers.

use crate::res::IoResult;
#[allow(unused_imports)]
use crate::rlock;

// Re-export
pub use crate::buf::cidx::ColumnIndex;
pub use crate::buf::opt::{BufferLocalOptions, FileEncodingOption};

use ahash::AHashMap as HashMap;
use ahash::AHashSet as HashSet;
use compact_str::CompactString;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use path_absolutize::Absolutize;
use ropey::{Rope, RopeBuilder};
use std::collections::BTreeMap;
use std::fs::Metadata;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};
use std::time::Instant;
use tracing::trace;

pub mod cidx;
pub mod opt;
pub mod unicode;

/// Buffer ID.
pub type BufferId = i32;

/// Next unique buffer ID.
///
/// NOTE: Start form 1.
pub fn next_buffer_id() -> BufferId {
  static VALUE: AtomicI32 = AtomicI32::new(1);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug)]
/// The Vim buffer, it is the in-memory texts mapping to the filesystem.
///
/// It contains several internal data:
/// 1. File name that associated with filesystem.
/// 2. File contents.
/// 3. File metadata.
///
/// To stable and avoid data racing issues, all file IO operations are made in pure blocking and
/// single-threading manner. And buffer also provide a set of APIs that serves as middle-level
/// primitives which can be used to implement high-level Vim ex commands, etc.
pub struct Buffer {
  id: BufferId,
  rope: Rope,
  rope_lines_width: BTreeMap<usize, ColumnIndex>,
  options: BufferLocalOptions,
  filename: Option<PathBuf>,
  absolute_filename: Option<PathBuf>,
  metadata: Option<Metadata>,
  last_sync_time: Option<Instant>,
}

pub type BufferArc = Arc<RwLock<Buffer>>;
pub type BufferWk = Weak<RwLock<Buffer>>;
pub type BufferReadGuard<'a> = RwLockReadGuard<'a, Buffer>;
pub type BufferWriteGuard<'a> = RwLockWriteGuard<'a, Buffer>;

impl Buffer {
  /// NOTE: This API should not be used to create new buffer, please use [`BuffersManager`] APIs to
  /// manage buffer instances.
  pub fn _new(
    rope: Rope,
    options: BufferLocalOptions,
    filename: Option<PathBuf>,
    absolute_filename: Option<PathBuf>,
    metadata: Option<Metadata>,
    last_sync_time: Option<Instant>,
  ) -> Self {
    Self {
      id: next_buffer_id(),
      rope,
      rope_lines_width: BTreeMap::new(),
      options,
      filename,
      absolute_filename,
      metadata,
      last_sync_time,
    }
  }

  #[cfg(test)]
  /// NOTE: This API should only be used for testing.
  pub fn _new_empty(options: BufferLocalOptions) -> Self {
    Self {
      id: next_buffer_id(),
      rope: Rope::new(),
      rope_lines_width: BTreeMap::new(),
      options,
      filename: None,
      absolute_filename: None,
      metadata: None,
      last_sync_time: None,
    }
  }

  pub fn to_arc(b: Buffer) -> BufferArc {
    Arc::new(RwLock::new(b))
  }

  pub fn id(&self) -> BufferId {
    self.id
  }

  pub fn filename(&self) -> &Option<PathBuf> {
    &self.filename
  }

  pub fn set_filename(&mut self, filename: Option<PathBuf>) {
    self.filename = filename;
  }

  pub fn absolute_filename(&self) -> &Option<PathBuf> {
    &self.absolute_filename
  }

  pub fn set_absolute_filename(&mut self, absolute_filename: Option<PathBuf>) {
    self.absolute_filename = absolute_filename;
  }

  pub fn metadata(&self) -> &Option<Metadata> {
    &self.metadata
  }

  pub fn set_metadata(&mut self, metadata: Option<Metadata>) {
    self.metadata = metadata;
  }

  pub fn last_sync_time(&self) -> &Option<Instant> {
    &self.last_sync_time
  }

  pub fn set_last_sync_time(&mut self, last_sync_time: Option<Instant>) {
    self.last_sync_time = last_sync_time;
  }
}

// Unicode {
impl Buffer {
  /// Get the display width for a `char`, supports both ASCI control codes and unicode.
  ///
  /// The char display width follows the
  /// [Unicode Standard Annex #11](https://www.unicode.org/reports/tr11/).
  pub fn char_width(&self, c: char) -> usize {
    unicode::char_width(&self.options, c)
  }

  /// Get the printable cell symbol and its display width.
  pub fn char_symbol(&self, c: char) -> (CompactString, usize) {
    unicode::char_symbol(&self.options, c)
  }
}
// Unicode }

// Rope {
impl Buffer {
  /// Get rope.
  pub fn get_rope(&self) -> &Rope {
    &self.rope
  }

  /// Get mutable rope.
  pub fn get_rope_mut(&mut self) -> &mut Rope {
    &mut self.rope
  }

  // /// Similar with [`Buffer::get_line`], but collect and clone a normal string with start index
  // /// (`start_char_idx`) and max chars length (`max_chars`).
  // /// NOTE: This is for performance reason that this API limits the max chars instead of the whole
  // /// line, this is useful for super long lines.
  // pub fn clone_line(
  //   &self,
  //   line_idx: usize,
  //   start_char_idx: usize,
  //   max_chars: usize,
  // ) -> Option<String> {
  //   match self.rope.get_line(line_idx) {
  //     Some(line) => match line.get_chars_at(start_char_idx) {
  //       Some(chars_iter) => {
  //         let mut builder = String::with_capacity(max_chars);
  //         for (i, c) in chars_iter.enumerate() {
  //           if i >= max_chars {
  //             return Some(builder);
  //           }
  //           builder.push(c);
  //         }
  //         Some(builder)
  //       }
  //       None => None,
  //     },
  //     None => None,
  //   }
  // }
}
// Rope }

// Options {
impl Buffer {
  pub fn options(&self) -> &BufferLocalOptions {
    &self.options
  }

  pub fn set_options(&mut self, options: &BufferLocalOptions) {
    self.options = options.clone();
  }
}
// Options }

// Display Width {
impl Buffer {
  /// See [`ColumnIndex::width_before`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn width_before(&mut self, line_idx: usize, char_idx: usize) -> usize {
    self.rope_lines_width.entry(line_idx).or_default();
    let rope_line = self.rope.line(line_idx);
    self
      .rope_lines_width
      .get_mut(&line_idx)
      .unwrap()
      .width_before(&self.options, &rope_line, char_idx)
  }

  /// See [`ColumnIndex::width_at`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn width_at(&mut self, line_idx: usize, char_idx: usize) -> usize {
    self.rope_lines_width.entry(line_idx).or_default();
    let rope_line = self.rope.line(line_idx);
    self
      .rope_lines_width
      .get_mut(&line_idx)
      .unwrap()
      .width_at(&self.options, &rope_line, char_idx)
  }

  /// See [`ColumnIndex::char_before`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_before(&mut self, line_idx: usize, width: usize) -> Option<usize> {
    self.rope_lines_width.entry(line_idx).or_default();
    let rope_line = self.rope.line(line_idx);
    self
      .rope_lines_width
      .get_mut(&line_idx)
      .unwrap()
      .char_before(&self.options, &rope_line, width)
  }

  /// See [`ColumnIndex::char_at`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_at(&mut self, line_idx: usize, width: usize) -> Option<usize> {
    self.rope_lines_width.entry(line_idx).or_default();
    let rope_line = self.rope.line(line_idx);
    self
      .rope_lines_width
      .get_mut(&line_idx)
      .unwrap()
      .char_at(&self.options, &rope_line, width)
  }

  /// See [`ColumnIndex::char_after`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_after(&mut self, line_idx: usize, width: usize) -> Option<usize> {
    self.rope_lines_width.entry(line_idx).or_default();
    let rope_line = self.rope.line(line_idx);
    self
      .rope_lines_width
      .get_mut(&line_idx)
      .unwrap()
      .char_after(&self.options, &rope_line, width)
  }

  /// See [`ColumnIndex::last_char_until`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn last_char_until(&mut self, line_idx: usize, width: usize) -> Option<usize> {
    self.rope_lines_width.entry(line_idx).or_default();
    let rope_line = self.rope.line(line_idx);
    self
      .rope_lines_width
      .get_mut(&line_idx)
      .unwrap()
      .last_char_until(&self.options, &rope_line, width)
  }

  /// See [`ColumnIndex::truncate_since_char`].
  pub fn truncate_line_since_char(&mut self, line_idx: usize, char_idx: usize) {
    self.rope_lines_width.entry(line_idx).or_default();
    self
      .rope_lines_width
      .get_mut(&line_idx)
      .unwrap()
      .truncate_since_char(char_idx)
  }

  /// See [`ColumnIndex::truncate_since_width`].
  pub fn truncate_line_since_width(&mut self, line_idx: usize, width: usize) {
    self.rope_lines_width.entry(line_idx).or_default();
    self
      .rope_lines_width
      .get_mut(&line_idx)
      .unwrap()
      .truncate_since_width(width)
  }

  /// Truncate lines at the tail, start from specified line index.
  pub fn truncate(&mut self, start_line_idx: usize) {
    self.rope_lines_width.retain(|&l, _| l < start_line_idx);
  }

  /// Remove one specified line.
  pub fn remove(&mut self, line_idx: usize) {
    self.rope_lines_width.remove(&line_idx);
  }

  /// Retain multiple specified lines.
  pub fn retain(&mut self, lines_idx: HashSet<usize>) {
    self.rope_lines_width.retain(|l, _| lines_idx.contains(l));
  }

  /// Clear.
  pub fn clear(&mut self) {
    self.rope_lines_width.clear()
  }
}
// Display Width }

#[derive(Debug, Clone)]
/// The manager for all normal (file) buffers.
///
/// NOTE: A buffer has its unique filepath (on filesystem), and there is at most 1 unnamed buffer.
pub struct BuffersManager {
  // Buffers collection
  buffers: BTreeMap<BufferId, BufferArc>,

  // Buffers maps by absolute file path.
  buffers_by_path: HashMap<Option<PathBuf>, BufferArc>,

  // Global-local options for buffers.
  global_local_options: BufferLocalOptions,
}

impl BuffersManager {
  pub fn new() -> Self {
    BuffersManager {
      buffers: BTreeMap::new(),
      buffers_by_path: HashMap::new(),
      global_local_options: BufferLocalOptions::default(),
    }
  }

  pub fn to_arc(b: BuffersManager) -> BuffersManagerArc {
    Arc::new(RwLock::new(b))
  }

  /// Open a file with a newly created buffer.
  ///
  /// The file name must be unique and not existed, there are two use cases:
  /// 1. If the file exists on filesystem, the buffer will read the file contents into buffer.
  /// 2. If the file doesn't exist, the buffer will be empty but only set the file name.
  ///
  /// # Returns
  ///
  /// It returns the buffer ID if the buffer created successfully, also the reading operations must
  /// be successful if the file exists on filesystem.
  /// Otherwise it returns the error.
  ///
  /// # Panics
  ///
  /// If the file name already exists.
  ///
  /// NOTE: This is a primitive API.
  pub fn new_file_buffer(&mut self, filename: &Path) -> IoResult<BufferId> {
    let abs_filename = match filename.absolutize() {
      Ok(abs_filename) => abs_filename.to_path_buf(),
      Err(e) => {
        trace!("Failed to absolutize filepath {:?}:{:?}", filename, e);
        return Err(e);
      }
    };

    assert!(!self
      .buffers_by_path
      .contains_key(&Some(abs_filename.clone())));

    let existed = match std::fs::exists(abs_filename.clone()) {
      Ok(existed) => existed,
      Err(e) => {
        trace!("Failed to detect file {:?}:{:?}", filename, e);
        return Err(e);
      }
    };

    let buf = if existed {
      match self.edit_file(filename, &abs_filename) {
        Ok(buf) => buf,
        Err(e) => {
          return Err(e);
        }
      }
    } else {
      Buffer::_new(
        Rope::new(),
        self.global_local_options().clone(),
        Some(filename.to_path_buf()),
        Some(abs_filename.clone()),
        None,
        None,
      )
    };

    let buf_id = buf.id();
    let buf = Buffer::to_arc(buf);
    self.buffers.insert(buf_id, buf.clone());
    self.buffers_by_path.insert(Some(abs_filename), buf);
    Ok(buf_id)
  }

  /// Create new empty buffer without file name.
  ///
  /// The file name of this buffer is empty, i.e. the buffer is unnamed.
  ///
  /// # Returns
  ///
  /// It returns the buffer ID if there is no other unnamed buffers.
  ///
  /// # Panics
  ///
  /// If there is already other unnamed buffers.
  ///
  /// NOTE: This is a primitive API.
  pub fn new_empty_buffer(&mut self) -> BufferId {
    assert!(!self.buffers_by_path.contains_key(&None));

    let buf = Buffer::_new(
      Rope::new(),
      self.global_local_options().clone(),
      None,
      None,
      None,
      None,
    );
    let buf_id = buf.id();
    let buf = Buffer::to_arc(buf);
    self.buffers.insert(buf_id, buf.clone());
    self.buffers_by_path.insert(None, buf);
    buf_id
  }

  #[cfg(test)]
  /// NOTE: This API should only be used for testing.
  pub fn _add_buffer(&mut self, buf: BufferArc) -> BufferId {
    let (buf_id, abs_filepath) = {
      let buf = rlock!(buf);
      (buf.id(), buf.absolute_filename().clone())
    };
    self.buffers.insert(buf_id, buf.clone());
    if abs_filepath.is_none() {
      assert!(!self.buffers_by_path.contains_key(&None));
    }
    self.buffers_by_path.insert(abs_filepath, buf);
    buf_id
  }
}

// Primitive APIs {

impl BuffersManager {
  fn to_rope(&self, buf: &[u8], bufsize: usize) -> Rope {
    let bufstr = self.to_str(buf, bufsize);
    let mut block = RopeBuilder::new();
    block.append(&bufstr.to_owned());
    block.finish()
  }

  fn to_str(&self, buf: &[u8], bufsize: usize) -> String {
    let fencoding = self.global_local_options().file_encoding();
    match fencoding {
      FileEncodingOption::Utf8 => String::from_utf8_lossy(&buf[0..bufsize]).into_owned(),
    }
  }

  // Implementation for [new_buffer_edit_file](new_buffer_edit_file).
  fn edit_file(&self, filename: &Path, absolute_filename: &Path) -> IoResult<Buffer> {
    match std::fs::File::open(filename) {
      Ok(fp) => {
        let metadata = match fp.metadata() {
          Ok(metadata) => metadata,
          Err(e) => {
            trace!("Failed to fetch metadata from file {:?}:{:?}", filename, e);
            return Err(e);
          }
        };
        let mut buf: Vec<u8> = Vec::new();
        let mut reader = std::io::BufReader::new(fp);
        let bytes = match reader.read_to_end(&mut buf) {
          Ok(bytes) => bytes,
          Err(e) => {
            trace!("Failed to read file {:?}:{:?}", filename, e);
            return Err(e);
          }
        };
        trace!(
          "Read {} bytes (buf: {}) from file {:?}",
          bytes,
          buf.len(),
          filename
        );
        assert!(bytes == buf.len());

        Ok(Buffer::_new(
          self.to_rope(&buf, buf.len()),
          self.global_local_options().clone(),
          Some(filename.to_path_buf()),
          Some(absolute_filename.to_path_buf()),
          Some(metadata),
          Some(Instant::now()),
        ))
      }
      Err(e) => {
        trace!("Failed to open file {:?}:{:?}", filename, e);
        Err(e)
      }
    }
  }
}

// Primitive APIs }

// BTreeMap {
impl BuffersManager {
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

  pub fn keys(&self) -> BuffersManagerKeys {
    self.buffers.keys()
  }

  pub fn values(&self) -> BuffersManagerValues {
    self.buffers.values()
  }

  pub fn iter(&self) -> BuffersManagerIter {
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

impl Default for BuffersManager {
  fn default() -> Self {
    BuffersManager::new()
  }
}

// Options {
impl BuffersManager {
  pub fn global_local_options(&self) -> &BufferLocalOptions {
    &self.global_local_options
  }

  pub fn global_local_options_mut(&mut self) -> &mut BufferLocalOptions {
    &mut self.global_local_options
  }

  pub fn set_global_local_options(&mut self, options: &BufferLocalOptions) {
    self.global_local_options = options.clone();
  }
}
// Options }

pub type BuffersManagerArc = Arc<RwLock<BuffersManager>>;
pub type BuffersManagerWk = Weak<RwLock<BuffersManager>>;
pub type BuffersManagerKeys<'a> = std::collections::btree_map::Keys<'a, BufferId, BufferArc>;
pub type BuffersManagerValues<'a> = std::collections::btree_map::Values<'a, BufferId, BufferArc>;
pub type BuffersManagerIter<'a> = std::collections::btree_map::Iter<'a, BufferId, BufferArc>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn next_buffer_id1() {
    assert!(next_buffer_id() > 0);
  }
}
