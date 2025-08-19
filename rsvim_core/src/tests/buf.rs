//! Buffer utils for testing.

#![allow(unused_imports)]

use crate::buf::opt::BufferOptions;
use crate::buf::text::Text;
use crate::buf::{Buffer, BufferArc, BuffersManager, BuffersManagerArc};
use crate::prelude::*;

use ropey::{Rope, RopeBuilder, RopeSlice};
use std::fs::File;
use std::io::BufReader;

pub fn make_buffer_from_lines(
  terminal_size: U16Size,
  opts: BufferOptions,
  lines: Vec<&str>,
) -> BufferArc {
  let mut rpb: RopeBuilder = RopeBuilder::new();
  for line in lines.iter() {
    rpb.append(line);
  }
  let rp = rpb.finish();
  let buf = Buffer::_new(opts, terminal_size, rp, None, None, None, None);
  Buffer::to_arc(buf)
}

pub fn make_empty_buffer(
  terminal_size: U16Size,
  opts: BufferOptions,
) -> BufferArc {
  let buf =
    Buffer::_new(opts, terminal_size, Rope::new(), None, None, None, None);
  Buffer::to_arc(buf)
}

pub fn make_buffers_manager(
  opts: BufferOptions,
  bufs: Vec<BufferArc>,
) -> BuffersManagerArc {
  let mut bm = BuffersManager::new();
  bm.set_global_local_options(&opts);
  for buf in bufs.iter() {
    bm._add_buffer(buf.clone());
  }
  BuffersManager::to_arc(bm)
}
