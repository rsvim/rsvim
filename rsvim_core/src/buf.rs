//! Vim buffers.

pub mod opt;
pub mod text;
pub mod unicode;

#[cfg(test)]
mod opt_tests;
#[cfg(test)]
mod text_tests;
#[cfg(test)]
mod unicode_tests;

use crate::prelude::*;
use opt::*;
use path_absolutize::Absolutize;
use ropey::Rope;
use ropey::RopeBuilder;
use std::fs::Metadata;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering;
use std::time::Instant;
use text::Text;

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
    opts: BufferOptions,
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

  pub fn options(&self) -> &BufferOptions {
    self.text.options()
  }

  pub fn set_options(&mut self, options: &BufferOptions) {
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

impl Buffer {
  pub fn has_filename(&self) -> bool {
    debug_assert_eq!(self.filename.is_some(), self.absolute_filename.is_some());
    self.filename.is_some() && self.absolute_filename.is_some()
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
  buffers_by_path: FoldMap<Option<PathBuf>, BufferArc>,

  // Global-local options for buffers.
  global_local_options: BufferOptions,
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
      buffers_by_path: FoldMap::new(),
      global_local_options: BufferOptionsBuilder::default().build().unwrap(),
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

    let existed = match std::fs::exists(&abs_filename) {
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

  /// Write (save) a buffer to filesystem.
  pub fn write_buffer(&self, buf_id: BufferId) -> TheResult<usize> {
    match self.buffers.get(&buf_id) {
      Some(buf) => {
        let mut buf = lock!(buf);
        if !buf.has_filename() {
          bail!(TheError::BufferHaveNoFileName(buf_id));
        }
        self.write_file(&mut buf)
      }
      None => {
        bail!(TheError::BufferNotFound(buf_id));
      }
    }
  }

  #[cfg(test)]
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

const BUF_PAGE_SIZE: usize = 8192_usize;

impl BuffersManager {
  fn edit_file(
    &self,
    canvas_size: U16Size,
    filename: &Path,
    absolute_filename: &Path,
  ) -> IoResult<Buffer> {
    match std::fs::File::open(filename) {
      Ok(fp) => {
        let metadata = fp.metadata().unwrap();
        let mut data: [u8; BUF_PAGE_SIZE] = [0_u8; BUF_PAGE_SIZE];
        let mut rope_builder = RopeBuilder::new();
        let fencoding = self.global_local_options().file_encoding();
        let mut bytes = 0_usize;
        let mut reader = std::io::BufReader::new(fp);
        loop {
          match reader.read(&mut data) {
            Ok(readded) => {
              debug_assert!(readded <= BUF_PAGE_SIZE);
              if readded == 0 {
                break;
              }
              bytes += readded;
              let payload = match fencoding {
                FileEncodingOption::Utf8 => {
                  String::from_utf8_lossy(&data[0..readded])
                }
              };
              rope_builder.append(&payload);
            }
            Err(e) => {
              error!("Failed to read file {:?}:{:?}", filename, e);
              return Err(e);
            }
          }
        }
        let rope = rope_builder.finish();

        trace!(
          "Read {} bytes (data: {}) from file {:?}",
          bytes,
          data.len(),
          filename
        );

        Ok(Buffer::_new(
          *self.global_local_options(),
          canvas_size,
          rope,
          Some(filename.to_path_buf()),
          Some(absolute_filename.to_path_buf()),
          Some(metadata),
          Some(Instant::now()),
        ))
      }
      Err(e) => {
        error!("Failed to open file {:?}:{:?}", filename, e);
        Err(e)
      }
    }
  }

  fn write_file(&self, buf: &mut Buffer) -> TheResult<usize> {
    let buf_id = buf.id();
    let filename = buf.filename().as_ref().unwrap();
    let abs_filename = buf.absolute_filename().as_ref().unwrap();

    let written_bytes = match std::fs::OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(abs_filename)
    {
      Ok(fp) => {
        let mut writer = std::io::BufWriter::new(fp);
        let payload = buf.text().rope().to_string();
        let mut data: Vec<u8> = Vec::with_capacity(payload.len());

        let n = match data.write(payload.as_bytes()) {
          Ok(n) => match writer.write_all(&data) {
            Ok(_) => match writer.flush() {
              Ok(_) => n,
              Err(e) => {
                bail!(TheError::WriteBufferFailed(buf_id, e));
              }
            },
            Err(e) => {
              bail!(TheError::WriteBufferFailed(buf_id, e));
            }
          },
          Err(e) => {
            bail!(TheError::WriteBufferFailed(buf_id, e));
          }
        };
        trace!("Write file {:?}, bytes: {:?}", filename, n);

        let fp1 = writer.get_ref();
        let metadata = fp1.metadata().unwrap();
        buf.set_metadata(Some(metadata));
        buf.set_last_sync_time(Some(Instant::now()));

        n
      }
      Err(e) => {
        bail!(TheError::WriteBufferOpenFailed(
          filename.to_string_lossy().to_string(),
          e
        ));
      }
    };

    Ok(written_bytes)
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
  pub fn global_local_options(&self) -> &BufferOptions {
    &self.global_local_options
  }

  pub fn global_local_options_mut(&mut self) -> &mut BufferOptions {
    &mut self.global_local_options
  }

  pub fn set_global_local_options(&mut self, options: &BufferOptions) {
    self.global_local_options = *options;
  }
}
// Options }
