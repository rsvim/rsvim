//! Buffer utils for testing.

use crate::buf::{Buffer, BufferArc};
use ropey::{Rope, RopeBuilder};
use std::fs::File;
use std::io::BufReader;

/// Create buffer from filename.
pub fn make_buffer_from_file(filename: String) -> BufferArc {
  let rop: Rope = Rope::from_reader(BufReader::new(File::open(filename).unwrap())).unwrap();
  let buf: Buffer = Buffer::from(rop);
  Buffer::to_arc(buf)
}

/// Create buffer from lines.
pub fn make_buffer_from_lines(lines: Vec<&str>) -> BufferArc {
  let mut rop: RopeBuilder = RopeBuilder::new();
  for line in lines.iter() {
    rop.append(line);
  }
  let buf: Buffer = Buffer::from(rop);
  Buffer::to_arc(buf)
}

/// Create empty buffer.
pub fn make_empty_buffer() -> BufferArc {
  let buf: Buffer = RopeBuilder::new().into();
  Buffer::to_arc(buf)
}
