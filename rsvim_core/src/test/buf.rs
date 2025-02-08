//! Buffer utils for testing.

use crate::buf::unicode;
use crate::buf::{Buffer, BufferArc, BufferLocalOptions};

use ropey::{Rope, RopeBuilder, RopeSlice};
use std::fs::File;
use std::io::BufReader;
use tracing::info;

/// Create rope from filename.
pub fn make_rope_from_file(filename: String) -> Rope {
  Rope::from_reader(BufReader::new(File::open(filename).unwrap())).unwrap()
}

/// Create rope from lines.
pub fn make_rope_from_lines(lines: Vec<&str>) -> Rope {
  let mut rb: RopeBuilder = RopeBuilder::new();
  for line in lines.iter() {
    rb.append(line);
  }
  rb.finish()
}

/// Create buffer from filename.
pub fn make_buffer_from_file(filename: String) -> BufferArc {
  let rp = make_rope_from_file(filename);
  let bf = Buffer::_new(rp, BufferLocalOptions::default(), None, None, None, None);
  Buffer::to_arc(bf)
}

/// Create buffer from lines.
pub fn make_buffer_from_lines(lines: Vec<&str>) -> BufferArc {
  let rp = make_rope_from_lines(lines);
  let buf = Buffer::_new(rp, BufferLocalOptions::default(), None, None, None, None);
  Buffer::to_arc(buf)
}

/// Create empty buffer.
pub fn make_empty_buffer() -> BufferArc {
  let buf = Buffer::_new_empty(BufferLocalOptions::default());
  Buffer::to_arc(buf)
}

pub fn print_buffer_line_details(options: &BufferLocalOptions, line: &RopeSlice, msg: &str) {
  let n = line.len_chars();
  if !msg.is_empty() {
    info!("line: {}", msg);
  } else {
    info!("line");
  }
  let mut builder = String::with_capacity(n);
  for c in line.chars() {
    builder.push(c);
  }
  info!("-{}-", builder);

  let mut builder = String::with_capacity(n);
  for (i, _) in line.chars().enumerate() {
    if i % 10 == 0 {
      builder.push_str(&format!("{}", i));
    } else if builder.len() < i {
      let diff = i - builder.len();
      builder.push_str(&" ".repeat(diff));
    }
  }
  info!("-{}- total:{}", builder, n);

  let mut builder = String::with_capacity(n);
  let mut w = 0_usize;
  for (_i, c) in line.chars().enumerate() {
    let cw = unicode::char_width(options, c);
    w += cw;
    if w == 1 || w % 10 == 0 {
      builder.push_str(&format!("{}", w));
    } else if builder.len() < w {
      let diff = w - builder.len();
      builder.push_str(&" ".repeat(diff));
    }
  }
  info!("-{}- display width total:{}", builder, w);

  let mut builder = String::with_capacity(n);
  let mut w = 0_usize;
  for (_i, c) in line.chars().enumerate() {
    let cw = unicode::char_width(options, c);
    w += cw;
    if cw > 1 {
      builder.push_str(&format!("{}", w));
    } else if builder.len() < w {
      let diff = w - builder.len();
      builder.push_str(&" ".repeat(diff));
    }
  }
  info!("-{}- width >=1 chars", builder);
}
