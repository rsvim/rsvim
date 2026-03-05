//! Buffer utils for testing.

use crate::buf::Buffer;
use crate::buf::BufferArc;
use crate::buf::BufferManager;
use crate::buf::BufferManagerArc;
use crate::buf::opt::BufferOptions;
use crate::hl::ColorScheme;
use crate::prelude::*;
use crate::syntax::Syntax;
use compact_str::ToCompactString;
use path_absolutize::Absolutize;
use ropey::Rope;
use ropey::RopeBuilder;
use std::path::PathBuf;

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
  let buf = Buffer::_new(
    opts,
    terminal_size,
    rp,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
  );
  Buffer::to_arc(buf)
}

pub fn make_buffer_from_lines_and_syntax(
  terminal_size: U16Size,
  opts: BufferOptions,
  lines: Vec<&str>,
  filename: PathBuf,
  syntax: Syntax,
  colorscheme: ColorScheme,
) -> BufferArc {
  let mut rpb: RopeBuilder = RopeBuilder::new();
  for line in lines.iter() {
    rpb.append(line);
  }
  let rp = rpb.finish();
  let file_extension = filename
    .extension()
    .map(|e| e.to_string_lossy().to_compact_string());
  let absolute_filename = filename.absolutize().unwrap().to_path_buf();
  let buf = Buffer::_new(
    opts,
    terminal_size,
    rp,
    Some(filename),
    file_extension,
    Some(absolute_filename),
    None,
    None,
    Some(syntax),
    Some(colorscheme),
  );
  Buffer::to_arc(buf)
}

pub fn make_empty_buffer(
  terminal_size: U16Size,
  opts: BufferOptions,
) -> BufferArc {
  let buf = Buffer::_new(
    opts,
    terminal_size,
    Rope::new(),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
  );
  Buffer::to_arc(buf)
}

pub fn make_buffers_manager(
  opts: BufferOptions,
  bufs: Vec<BufferArc>,
) -> BufferManagerArc {
  let mut bm = BufferManager::new();
  bm.set_global_local_options(&opts);
  for buf in bufs.iter() {
    bm._add_buffer(buf.clone());
  }
  BufferManager::to_arc(bm)
}
