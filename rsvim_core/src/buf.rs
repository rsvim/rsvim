//! Vim buffers.

use crate::res::IoResult;

// Re-export
pub use crate::buf::opt::{BufferLocalOptions, FileEncoding};
pub use crate::buf::widx::{ColIndex, LineIndex};

use ahash::AHashMap as HashMap;
use compact_str::CompactString;
use parking_lot::RwLock;
use path_absolutize::Absolutize;
use ropey::iter::Lines;
use ropey::{Rope, RopeBuilder, RopeSlice};
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs::Metadata;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};
use std::time::Instant;
use tracing::trace;

pub mod opt;
pub mod unicode;
pub mod widx;

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
  width_index: LineIndex,
  options: BufferLocalOptions,
  filename: Option<PathBuf>,
  absolute_filename: Option<PathBuf>,
  metadata: Option<Metadata>,
  last_sync_time: Option<Instant>,
}

pub type BufferArc = Arc<RwLock<Buffer>>;
pub type BufferWk = Weak<RwLock<Buffer>>;

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
      width_index: LineIndex::new(),
      options,
      filename,
      absolute_filename,
      metadata,
      last_sync_time,
    }
  }

  /// NOTE: This API should not be used to create new buffer, please use [`BuffersManager`] APIs to
  /// manage buffer instances.
  pub fn _new_empty(options: BufferLocalOptions) -> Self {
    Self {
      id: next_buffer_id(),
      rope: Rope::new(),
      width_index: LineIndex::new(),
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

  // pub fn status(&self) -> BufferStatus {
  //   BufferStatus::INIT
  // }

  // pub fn worker_send_to_master(&self) -> &Sender<WorkerToMasterMessage> {
  //   &self.worker_send_to_master
  // }
}

// Unicode {
impl Buffer {
  /// Get the display width for a `char`, supports both ASCI control codes and unicode.
  ///
  /// The char display width follows the
  /// [Unicode Standard Annex #11](https://www.unicode.org/reports/tr11/), implemented with
  /// [UnicodeWidthChar], there's another equivalent crate
  /// [icu::properties::EastAsianWidth](https://docs.rs/icu/latest/icu/properties/maps/fn.east_asian_width.html#).
  pub fn char_width(&self, c: char) -> usize {
    unicode::char_width(&self.options, c)
  }

  /// Get the printable cell symbol and its display width.
  pub fn char_symbol(&self, c: char) -> (CompactString, usize) {
    unicode::char_symbol(&self.options, c)
  }

  /// Get the display width for a unicode `str`.
  pub fn str_width(&self, s: &str) -> usize {
    unicode::str_width(&self.options, s)
  }

  /// Get the printable cell symbols and the display width for a unicode `str`.
  pub fn str_symbols(&self, s: &str) -> (CompactString, usize) {
    unicode::str_symbols(&self.options, s)
  }
}
// Unicode }

// Rope {
impl Buffer {
  // lines {

  /// Same with [`Rope::get_line`](Rope::get_line).
  pub fn get_line(&self, line_idx: usize) -> Option<RopeSlice> {
    self.rope.get_line(line_idx)
  }

  /// Same with [`Rope::get_lines_at`](Rope::get_lines_at).
  pub fn get_lines_at(&self, line_idx: usize) -> Option<Lines> {
    self.rope.get_lines_at(line_idx)
  }

  /// Similar with [`Buffer::get_line`], but collect and clone a normal string with start index
  /// (`start_char_idx`) and max chars length (`max_chars`).
  /// NOTE: This is for performance reason that this API limits the max chars instead of the whole
  /// line, this is useful for super long lines.
  pub fn clone_line(
    &self,
    line_idx: usize,
    start_char_idx: usize,
    max_chars: usize,
  ) -> Option<String> {
    match self.rope.get_line(line_idx) {
      Some(line) => match line.get_chars_at(start_char_idx) {
        Some(chars_iter) => {
          let mut builder = String::new();
          builder.reserve(max_chars);
          let mut n = 0_usize;
          for c in chars_iter {
            if n >= max_chars {
              return Some(builder);
            }
            builder.push(c);
            n += 1;
          }
          Some(builder)
        }
        None => None,
      },
      None => None,
    }
  }

  /// Same with [`Rope::lines`](Rope::lines).
  pub fn lines(&self) -> Lines {
    self.rope.lines()
  }

  /// Same with [`Rope::len_lines`](Rope::len_lines).
  pub fn len_lines(&self) -> usize {
    self.rope.len_lines()
  }

  // lines }

  /// Alias to method [`Rope::write_to`](Rope::write_to).
  pub fn write_to<T: std::io::Write>(&self, writer: T) -> std::io::Result<()> {
    self.rope.write_to(writer)
  }

  /// Alias to method [`Rope::append`](Rope::append).
  pub fn append(&mut self, other: Rope) {
    self.rope.append(other)
  }
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

  pub fn tab_stop(&self) -> u16 {
    self.options.tab_stop()
  }

  pub fn set_tab_stop(&mut self, value: u16) {
    self.options.set_tab_stop(value);
  }
}
// Options }

// Display Width {
impl Buffer {
  /// See [`ColIndex::width_before`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn width_before(&mut self, line_idx: usize, char_idx: usize) -> usize {
    self
      .width_index
      .width_before(&self.options, &self.rope, line_idx, char_idx)
  }

  /// See [`ColIndex::width_until`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn width_until(&mut self, line_idx: usize, char_idx: usize) -> usize {
    self
      .width_index
      .width_until(&self.options, &self.rope, line_idx, char_idx)
  }

  /// See [`ColIndex::char_before`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_before(&mut self, line_idx: usize, width: usize) -> Option<usize> {
    self
      .width_index
      .char_before(&self.options, &self.rope, line_idx, width)
  }

  /// See [`ColIndex::char_until`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_until(&mut self, line_idx: usize, width: usize) -> Option<usize> {
    self
      .width_index
      .char_until(&self.options, &self.rope, line_idx, width)
  }

  /// See [`ColIndex::char_after`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_after(&mut self, line_idx: usize, width: usize) -> Option<usize> {
    self
      .width_index
      .char_after(&self.options, &self.rope, line_idx, width)
  }

  /// See [`ColIndex::last_char`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn last_char(&mut self, line_idx: usize) -> Option<usize> {
    self
      .width_index
      .last_char(&self.options, &self.rope, line_idx)
  }

  /// See [`ColIndex::truncate_by_char`].
  pub fn truncate_line_since_char(&mut self, line_idx: usize, char_idx: usize) {
    self
      .width_index
      .truncate_line_since_char(line_idx, char_idx)
  }

  /// See [`ColIndex::truncate_by_width`].
  pub fn truncate_line_since_width(&mut self, line_idx: usize, width: usize) {
    self.width_index.truncate_line_since_width(line_idx, width)
  }

  /// Truncate lines at the tail, start from specified line index.
  pub fn truncate(&mut self, start_line_idx: usize) {
    self.width_index.truncate(start_line_idx)
  }

  /// Remove one specified line.
  pub fn remove(&mut self, line_idx: usize) {
    self.width_index.remove(line_idx)
  }

  /// Retain multiple specified lines.
  pub fn retain(&mut self, lines_idx: HashSet<usize>) {
    self.width_index.retain(lines_idx)
  }

  /// Clear.
  pub fn clear(&mut self) {
    self.width_index.clear()
  }
}
// Display Width }

impl PartialEq for Buffer {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for Buffer {}

#[derive(Debug, Clone)]
/// The manager for all normal (file) buffers.
///
/// NOTE: A buffer has its unique filepath (on filesystem), and there is at most 1 unnamed buffer.
pub struct BuffersManager {
  // Buffers collection
  buffers: BTreeMap<BufferId, BufferArc>,

  // Buffers maps by absolute file path.
  buffers_by_path: HashMap<Option<PathBuf>, BufferArc>,

  // Local options for buffers.
  local_options: BufferLocalOptions,
}

impl BuffersManager {
  pub fn new() -> Self {
    BuffersManager {
      buffers: BTreeMap::new(),
      buffers_by_path: HashMap::new(),
      local_options: BufferLocalOptions::default(),
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
        self.local_options().clone(),
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
      self.local_options().clone(),
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
    let fencoding = self.local_options().file_encoding();
    match fencoding {
      FileEncoding::Utf8 => String::from_utf8_lossy(&buf[0..bufsize]).into_owned(),
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
          self.local_options().clone(),
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
  pub fn local_options(&self) -> &BufferLocalOptions {
    &self.local_options
  }

  pub fn set_local_options(&mut self, options: &BufferLocalOptions) {
    self.local_options = options.clone();
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
  // use std::fs::File;
  // use tempfile::tempfile;
  // use tokio::sync::mpsc::Receiver;

  // fn make_channel() -> (
  //   Sender<WorkerToMasterMessage>,
  //   Receiver<WorkerToMasterMessage>,
  // ) {
  //   tokio::sync::mpsc::channel(1)
  // }

  // #[test]
  // fn buffer_from1() {
  //   let (sender, _) = make_channel();
  //
  //   let r1 = Rope::from_str("Hello");
  //   let buf1 = Buffer::_from_rope(sender.clone(), r1);
  //   let tmp1 = tempfile().unwrap();
  //   buf1.write_to(tmp1).unwrap();
  //
  //   let r2 = Rope::from_reader(File::open("Cargo.toml").unwrap()).unwrap();
  //   let buf2 = Buffer::_from_rope(sender, r2);
  //   let tmp2 = tempfile().unwrap();
  //   buf2.write_to(tmp2).unwrap();
  // }
  //
  // #[test]
  // fn buffer_from2() {
  //   let (sender, _) = make_channel();
  //
  //   let mut builder1 = RopeBuilder::new();
  //   builder1.append("Hello");
  //   builder1.append("World");
  //   let buf1 = Buffer::_from_rope_builder(sender, builder1);
  //   let tmp1 = tempfile().unwrap();
  //   buf1.write_to(tmp1).unwrap();
  // }

  #[test]
  fn next_buffer_id1() {
    assert!(next_buffer_id() > 0);
  }

  // #[test]
  // fn buffer_unicode_width1() {
  //   let (sender, _) = make_channel();
  //
  //   let b1 = Buffer::_from_rope_builder(sender, RopeBuilder::new());
  //   assert_eq!(b1.char_width('A'), 1);
  //   assert_eq!(b1.char_symbol('A'), (CompactString::new("A"), 1));
  //   assert_eq!(b1.str_width("ABCDEFG"), 7);
  //   assert_eq!(
  //     b1.str_symbols("ABCDEFG"),
  //     (CompactString::new("ABCDEFG"), 7)
  //   );
  // }
}
