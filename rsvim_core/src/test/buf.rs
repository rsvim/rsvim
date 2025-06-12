//! Buffer utils for testing.

#![allow(unused_imports)]

use crate::buf::{Buffer, BufferArc, BufferLocalOptions, BuffersManager, BuffersManagerArc, Text};
//use crate::envar;
use crate::lock;

use ropey::{Rope, RopeBuilder, RopeSlice};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufReader;
use tracing::{self, info, trace};
use tracing_appender::non_blocking::DEFAULT_BUFFERED_LINES_LIMIT;

pub fn make_buffer_from_lines(
  terminal_height: u16,
  opts: BufferLocalOptions,
  lines: Vec<&str>,
) -> BufferArc {
  let mut rpb: RopeBuilder = RopeBuilder::new();
  for line in lines.iter() {
    rpb.append(line);
  }
  let rp = rpb.finish();
  let tx = Text::new(terminal_height, rp, opts);
  let buf = Buffer::_new(text, None, None, None, None);
  Buffer::to_arc(buf)
}

pub fn make_empty_buffer(terminal_height: u16, opts: BufferLocalOptions) -> BufferArc {
  let tx = Text::new(terminal_height, Rope::new(), opts);
  let buf = Buffer::_new(tx, None, None, None, None);
  Buffer::to_arc(buf)
}

pub fn make_buffers_manager(opts: BufferLocalOptions, bufs: Vec<BufferArc>) -> BuffersManagerArc {
  let mut bm = BuffersManager::new();
  bm.set_global_local_options(&opts);
  for buf in bufs.iter() {
    bm._add_buffer(buf.clone());
  }
  BuffersManager::to_arc(bm)
}
