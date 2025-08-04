//! Vim buffers.

use crate::prelude::*;

use opt::*;
use text::Text;

use path_absolutize::Absolutize;
use ropey::{Rope, RopeBuilder};
use std::collections::BTreeMap;
use std::fs::Metadata;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;

pub mod opt;
pub mod text;
pub mod unicode;

#[cfg(test)]
mod opt_tests;
#[cfg(test)]
mod text_tests;
#[cfg(test)]
mod unicode_tests;

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
  text: Text,
  filename: Option<PathBuf>,
  absolute_filename: Option<PathBuf>,
  metadata: Option<Metadata>,
  last_sync_time: Option<Instant>,
}

arc_mutex_ptr!(Buffer);

impl Buffer {
  /// NOTE: This API should not be used to create new buffer, please use [`BuffersManager`] APIs to
  /// manage buffer instances.
  pub fn _new(
    opts: BufferLocalOptions,
    canvas_size: U16Size,
    rope: Rope,
    filename: Option<PathBuf>,
    absolute_filename: Option<PathBuf>,
    metadata: Option<Metadata>,
    last_sync_time: Option<Instant>,
  ) -> Self {
    let text = Text::new(opts, canvas_size, rope);
    Self {
      id: next_buffer_id(),
      text,
      filename,
      absolute_filename,
      metadata,
      last_sync_time,
    }
  }

  pub fn id(&self) -> BufferId {
    self.id
  }

  pub fn text(&self) -> &Text {
    &self.text
  }

  pub fn text_mut(&mut self) -> &mut Text {
    &mut self.text
  }

  pub fn options(&self) -> &BufferLocalOptions {
    self.text.options()
  }

  pub fn set_options(&mut self, options: &BufferLocalOptions) {
    self.text.set_options(options);
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

arc_mutex_ptr!(BuffersManager);

pub type BuffersManagerKeys<'a> =
  std::collections::btree_map::Keys<'a, BufferId, BufferArc>;
pub type BuffersManagerValues<'a> =
  std::collections::btree_map::Values<'a, BufferId, BufferArc>;
pub type BuffersManagerIter<'a> =
  std::collections::btree_map::Iter<'a, BufferId, BufferArc>;

impl BuffersManager {
  pub fn new() -> Self {
    BuffersManager {
      buffers: BTreeMap::new(),
      buffers_by_path: HashMap::new(),
      global_local_options: BufferLocalOptionsBuilder::default()
        .build()
        .unwrap(),
    }
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
  pub fn new_file_buffer(
    &mut self,
    canvas_size: U16Size,
    filename: &Path,
  ) -> IoResult<BufferId> {
    let abs_filename = match filename.absolutize() {
      Ok(abs_filename) => abs_filename.to_path_buf(),
      Err(e) => {
        trace!("Failed to absolutize filepath {:?}:{:?}", filename, e);
        return Err(e);
      }
    };

    debug_assert!(
      !self
        .buffers_by_path
        .contains_key(&Some(abs_filename.clone()))
    );

    let existed = match std::fs::exists(abs_filename.clone()) {
      Ok(existed) => existed,
      Err(e) => {
        trace!("Failed to detect file {:?}:{:?}", filename, e);
        return Err(e);
      }
    };

    let buf = if existed {
      match self.edit_file(canvas_size, filename, &abs_filename) {
        Ok(buf) => buf,
        Err(e) => {
          return Err(e);
        }
      }
    } else {
      Buffer::_new(
        *self.global_local_options(),
        canvas_size,
        Rope::new(),
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
  pub fn new_empty_buffer(&mut self, canvas_size: U16Size) -> BufferId {
    debug_assert!(!self.buffers_by_path.contains_key(&None));

    let buf = Buffer::_new(
      *self.global_local_options(),
      canvas_size,
      Rope::new(),
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

  #[cfg(debug_assertions)]
  /// NOTE: This API should only be used for testing.
  pub fn _add_buffer(&mut self, buf: BufferArc) -> BufferId {
    let (buf_id, abs_filepath) = {
      let buf = lock!(buf);
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
      FileEncodingOption::Utf8 => {
        String::from_utf8_lossy(&buf[0..bufsize]).into_owned()
      }
    }
  }

  // Implementation for [new_buffer_edit_file](new_buffer_edit_file).
  fn edit_file(
    &self,
    canvas_size: U16Size,
    filename: &Path,
    absolute_filename: &Path,
  ) -> IoResult<Buffer> {
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
        debug_assert!(bytes == buf.len());

        Ok(Buffer::_new(
          *self.global_local_options(),
          canvas_size,
          self.to_rope(&buf, buf.len()),
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

  pub fn keys(&self) -> BuffersManagerKeys<'_> {
    self.buffers.keys()
  }

  pub fn values(&self) -> BuffersManagerValues<'_> {
    self.buffers.values()
  }

  pub fn iter(&self) -> BuffersManagerIter<'_> {
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
    self.global_local_options = *options;
  }
}
// Options }
