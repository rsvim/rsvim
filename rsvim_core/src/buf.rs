//! Vim buffers.

use crate::buf::opt::BufferLocalOptions;
use crate::defaults::grapheme::AsciiControlCodeFormatter;
use crate::envar;
use crate::evloop::msg::{ReadBytes, WorkerToMasterMessage};
use crate::res::{TheBufferErr, TheBufferResult};

use ascii::AsciiChar;
use compact_str::CompactString;
use parking_lot::RwLock;
use ropey::iter::Lines;
use ropey::{Rope, RopeBuilder, RopeSlice};
use std::collections::BTreeMap;
use std::convert::From;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error};
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
/// The Vim buffer's status.
pub enum BufferStatus {
  INIT,    // After created.
  LOADING, // Loading text content from disk file.
  SAVING,  // Saving buffer content to disk file.
  SYNCED,  // Synced content with file system.
  CHANGED, // Buffer content has been modified.
}

#[derive(Debug)]
/// The Vim buffer, it is the memory mapping to a file on hardware disk, i.e. file system.
///
/// It contains several features:
/// 1. It can be associated with a file, or detached with no file.
/// 2. When associated with a file, it can load from or save to the file. While detached with no
///    file, it cannot.
pub struct Buffer {
  id: BufferId,
  rope: Rope,
  options: BufferLocalOptions,
  filename: Option<String>,
  status: BufferStatus,
  worker_send_to_master: Sender<WorkerToMasterMessage>,
}

pub type BufferArc = Arc<RwLock<Buffer>>;
pub type BufferWk = Weak<RwLock<Buffer>>;

impl Buffer {
  /// Make buffer with default [`BufferLocalOptions`].
  pub fn new(worker_send_to_master: Sender<WorkerToMasterMessage>) -> Self {
    Buffer {
      id: next_buffer_id(),
      rope: Rope::new(),
      options: BufferLocalOptions::default(),
      filename: None,
      status: BufferStatus::INIT,
      worker_send_to_master,
    }
  }

  pub fn to_arc(b: Buffer) -> BufferArc {
    Arc::new(RwLock::new(b))
  }

  pub fn id(&self) -> BufferId {
    self.id
  }

  pub fn filename(&self) -> &Option<String> {
    &self.filename
  }

  pub fn status(&self) -> BufferStatus {
    self.status
  }
}

// Unicode {
impl Buffer {
  /// Get the display width for a unicode `char`.
  pub fn char_width(&self, c: char) -> usize {
    if c.is_ascii_control() {
      let ac = AsciiChar::from_ascii(c).unwrap();
      match ac {
        AsciiChar::Tab => self.tab_stop() as usize,
        AsciiChar::LineFeed | AsciiChar::CarriageReturn => 0,
        _ => {
          let ascii_formatter = AsciiControlCodeFormatter::from(ac);
          format!("{}", ascii_formatter).len()
        }
      }
    } else {
      UnicodeWidthChar::width_cjk(c).unwrap()
    }
  }

  /// Get the printable cell symbol and its display width.
  pub fn char_symbol(&self, c: char) -> (CompactString, usize) {
    let width = self.char_width(c);
    if c.is_ascii_control() {
      let ac = AsciiChar::from_ascii(c).unwrap();
      match ac {
        AsciiChar::Tab => (
          CompactString::from(" ".repeat(self.tab_stop() as usize)),
          width,
        ),
        AsciiChar::LineFeed | AsciiChar::CarriageReturn => (CompactString::new(""), width),
        _ => {
          let ascii_formatter = AsciiControlCodeFormatter::from(ac);
          (CompactString::from(format!("{}", ascii_formatter)), width)
        }
      }
    } else {
      (CompactString::from(c.to_string()), width)
    }
  }

  /// Get the display width for a unicode `str`.
  pub fn str_width(&self, s: &str) -> usize {
    s.chars().map(|c| self.char_width(c)).sum()
  }

  /// Get the printable cell symbols and the display width for a unicode `str`.
  pub fn str_symbols(&self, s: &str) -> (CompactString, usize) {
    s.chars().map(|c| self.char_symbol(c)).fold(
      (CompactString::with_capacity(s.len()), 0_usize),
      |(mut init_symbol, init_width), (mut symbol, width)| {
        init_symbol.push_str(symbol.as_mut_str());
        (init_symbol, init_width + width)
      },
    )
  }
}
// Unicode }

fn into_rope(buf: &[u8], bufsize: usize) -> Rope {
  let bufstr = into_str(buf, bufsize);
  let mut block = RopeBuilder::new();
  block.append(&bufstr.to_owned());
  block.finish()
}

fn into_str(buf: &[u8], bufsize: usize) -> String {
  String::from_utf8_lossy(&buf[0..bufsize]).into_owned()
}

// Rope {
impl Buffer {
  /// Alias to method [`Rope::get_line`](Rope::get_line).
  pub fn get_line(&self, line_idx: usize) -> Option<RopeSlice> {
    self.rope.get_line(line_idx)
  }

  /// Alias to method [`Rope::get_lines_at`](Rope::get_lines_at).
  pub fn get_lines_at(&self, line_idx: usize) -> Option<Lines> {
    self.rope.get_lines_at(line_idx)
  }

  /// Alias to method [`Rope::lines`](Rope::lines).
  pub fn lines(&self) -> Lines {
    self.rope.lines()
  }

  /// Alias to method [`Rope::write_to`](Rope::write_to).
  pub fn write_to<T: std::io::Write>(&self, writer: T) -> std::io::Result<()> {
    self.rope.write_to(writer)
  }

  /// Alias to method [`Rope::append`](Rope::append).
  pub fn append(&mut self, other: Rope) {
    self.rope.append(other)
  }

  /// Bind buffer to a file.
  ///
  /// If current buffer is initialized, it will immediately start loading
  /// If current buffer is already associated, it will be renamed to new filename (only if the file
  /// not exists).
  ///
  /// Returns
  ///
  /// - It returns the newest status and loaded bytes after reading complete (if successful).
  /// - Otherwise it returns [`TheBufferErr`](crate::res::TheBufferErr) to indicate some errors.
  pub async fn bind(&mut self, filename: &str) -> TheBufferResult<(BufferStatus, usize)> {
    match &self.filename {
      Some(associated) => {
        // Associated
        assert!(
          self.status == BufferStatus::SYNCED
            || self.status == BufferStatus::LOADING
            || self.status == BufferStatus::SAVING
            || self.status == BufferStatus::CHANGED
        );
        if associated == filename {
          // The same file
          return Err(TheBufferErr::BufferAlreadyBinded(associated.to_string()));
        }

        let file_exists = match tokio::fs::try_exists(filename).await {
          Ok(exists) => exists,
          Err(e) => return Err(TheBufferErr::IoErr(e)),
        };

        if file_exists {
          // File already exists
          return Err(TheBufferErr::FileAlreadyExists(filename.to_string()));
        } else {
          // File not exists
          // Rename buffer, or say, bind to new filename, change status to `CHANGED`.
          self.filename = Some(filename.to_string());
          self.status = BufferStatus::CHANGED;
          return Ok((self.status, 0_usize));
        }
      }
      None => {
        // Detached
        assert!(self.status == BufferStatus::INIT || self.status == BufferStatus::CHANGED);

        match tokio::fs::File::open(filename).await {
          Ok(mut fp) => {
            // Rename buffer, or say, bind to new filename, start `LOADING`.
            self.filename = Some(filename.to_string());
            self.status = BufferStatus::LOADING;

            let mut total_read_bytes = 0_usize;
            let mut iobuf: Vec<u8> = vec![0_u8; envar::IO_BUF_SIZE()];

            loop {
              match fp.read(&mut iobuf).await {
                Ok(n) => {
                  debug!("Read {} bytes: {:?}", n, into_str(&iobuf, n));

                  // Load into buffer.
                  self.append(into_rope(&iobuf, n));

                  // After read each block, immediately notify main thread so UI tree can render
                  // it on terminal.
                  debug!("Notify master after each block read");
                  self
                    .worker_send_to_master
                    .send(WorkerToMasterMessage::ReadBytes(ReadBytes::new(n)))
                    .await
                    .unwrap();

                  if n == 0 {
                    // Finish reading, exit loop
                    break;
                  }
                  total_read_bytes += n;
                }
                Err(e) => {
                  // Unexpected error
                  error!("Error reading file: {:?}:{:?}", e.kind(), e.to_string());
                  return Err(TheBufferErr::IoErr(e));
                }
              }
            }

            self.status = BufferStatus::SYNCED;
            return Ok((self.status, total_read_bytes));
          }
          Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
              // File not found
              // Rename buffer, or say, bind to new filename, change status to `CHANGED`.
              self.filename = Some(filename.to_string());
              self.status = BufferStatus::CHANGED;
              return Ok((self.status, 0_usize));
            }
            _ => {
              error!("Error opening file: {:?}:{:?}", e.kind(), e.to_string());
              return Err(TheBufferErr::IoErr(e));
            }
          },
        };
      }
    }
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

impl Buffer {
  /// Make buffer from [`Rope`].
  fn from_rope(worker_send_to_master: Sender<WorkerToMasterMessage>, rope: Rope) -> Self {
    Buffer {
      id: next_buffer_id(),
      rope,
      options: BufferLocalOptions::default(),
      filename: None,
      status: BufferStatus::INIT,
      worker_send_to_master,
    }
  }

  /// Make buffer from [`RopeBuilder`].
  fn from_rope_builder(
    worker_send_to_master: Sender<WorkerToMasterMessage>,
    builder: RopeBuilder,
  ) -> Self {
    Buffer {
      id: next_buffer_id(),
      rope: builder.finish(),
      options: BufferLocalOptions::default(),
      filename: None,
      status: BufferStatus::INIT,
      worker_send_to_master,
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

  pub fn new_buffer(&mut self, worker_send_to_master: Sender<WorkerToMasterMessage>) -> BufferId {
    let mut buf = Buffer::new(worker_send_to_master);
    buf.set_options(self.local_options());
    let buf_id = buf.id();
    self.buffers.insert(buf_id, Buffer::to_arc(buf));
    buf_id
  }

  pub fn new_buffer_from_rope(
    &mut self,
    worker_send_to_master: Sender<WorkerToMasterMessage>,
    rope: Rope,
  ) -> BufferId {
    let mut buf = Buffer::from_rope(worker_send_to_master, rope);
    buf.set_options(self.local_options());
    let buf_id = buf.id();
    self.buffers.insert(buf_id, Buffer::to_arc(buf));
    buf_id
  }

  pub fn new_buffer_from_rope_builder(
    &mut self,
    worker_send_to_master: Sender<WorkerToMasterMessage>,
    rope_builder: RopeBuilder,
  ) -> BufferId {
    let mut buf = Buffer::from_rope_builder(worker_send_to_master, rope_builder);
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

  use tokio::sync::mpsc::Receiver;

  fn make_channel() -> (
    Sender<WorkerToMasterMessage>,
    Receiver<WorkerToMasterMessage>,
  ) {
    tokio::sync::mpsc::channel(1)
  }

  #[test]
  fn buffer_from1() {
    let (sender, _) = make_channel();

    let r1 = Rope::from_str("Hello");
    let buf1 = Buffer::from_rope(sender.clone(), r1);
    let tmp1 = tempfile().unwrap();
    buf1.write_to(tmp1).unwrap();

    let r2 = Rope::from_reader(File::open("Cargo.toml").unwrap()).unwrap();
    let buf2 = Buffer::from_rope(sender, r2);
    let tmp2 = tempfile().unwrap();
    buf2.write_to(tmp2).unwrap();
  }

  #[test]
  fn buffer_from2() {
    let (sender, _) = make_channel();

    let mut builder1 = RopeBuilder::new();
    builder1.append("Hello");
    builder1.append("World");
    let buf1 = Buffer::from_rope_builder(sender, builder1);
    let tmp1 = tempfile().unwrap();
    buf1.write_to(tmp1).unwrap();
  }

  #[test]
  fn next_buffer_id1() {
    assert!(next_buffer_id() > 0);
  }

  #[test]
  fn buffer_unicode_width1() {
    let (sender, _) = make_channel();

    let b1 = Buffer::from_rope_builder(sender, RopeBuilder::new());
    assert_eq!(b1.char_width('A'), 1);
    assert_eq!(b1.char_symbol('A'), (CompactString::new("A"), 1));
    assert_eq!(b1.str_width("ABCDEFG"), 7);
    assert_eq!(
      b1.str_symbols("ABCDEFG"),
      (CompactString::new("ABCDEFG"), 7)
    );
  }
}
