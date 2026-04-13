//! Buffer utils for testing.

use crate::buf::Buffer;
use crate::buf::BufferArc;
use crate::buf::BufferManager;
use crate::buf::BufferManagerArc;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::EndOfLineOption;
use crate::hl::ColorSchemeArc;
use crate::hl::ColorSchemeManager;
use crate::prelude::*;
use crate::syntax;
use crate::syntax::Syntax;
use crate::syntax::SyntaxEdit;
use crate::syntax::SyntaxEditNew;
use crate::syntax::SyntaxManager;
use assert_fs::NamedTempFile;
use compact_str::ToCompactString;
use path_absolutize::Absolutize;
use ropey::Rope;
use ropey::RopeBuilder;
use std::sync::Arc;
use tokio::time::Instant;

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

pub fn make_buffer_from_tmpfile_and_syntax(
  terminal_size: U16Size,
  opts: BufferOptions,
  tmpfile: &NamedTempFile,
  syntax: Syntax,
  colorscheme: ColorSchemeArc,
) -> BufferArc {
  let mut rpb: RopeBuilder = RopeBuilder::new();

  let filename = tmpfile.path();
  let absolute_filename = filename.absolutize().unwrap();
  let metadata = std::fs::metadata(&absolute_filename).unwrap();
  let file_content = std::fs::read_to_string(&absolute_filename).unwrap();
  let lines = file_content.split("\n").collect::<Vec<&str>>();

  let buf_eol = Into::<EndOfLineOption>::into(opts.file_format());
  for (i, line) in lines.iter().enumerate() {
    trace!("[{}]:{:?}", i, line);
    rpb.append(line);
    rpb.append(&format!("{}", buf_eol));
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
    Some(filename.to_path_buf()),
    file_extension,
    Some(absolute_filename),
    Some(metadata),
    Some(Instant::now()),
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
  let syntax_mgr = SyntaxManager::to_arc(SyntaxManager::new());
  let cs_mgr = ColorSchemeManager::to_arc(ColorSchemeManager::new());
  let mut bm =
    BufferManager::new(Arc::downgrade(&syntax_mgr), Arc::downgrade(&cs_mgr));
  bm.set_global_local_options(&opts);
  for buf in bufs.iter() {
    bm._add_buffer(buf.clone());
  }
  BufferManager::to_arc(bm)
}

pub fn make_syntax_and_colorscheme(
  tmpfile: &NamedTempFile,
) -> (Syntax, ColorSchemeArc) {
  let syntax_mgr = SyntaxManager::to_arc(SyntaxManager::new());
  let cs_mgr = ColorSchemeManager::to_arc(ColorSchemeManager::new());
  let buffer_manager =
    BufferManager::new(Arc::downgrade(&syntax_mgr), Arc::downgrade(&cs_mgr));

  let filename = tmpfile.path();
  let file_extension = filename
    .extension()
    .map(|e| e.to_string_lossy().to_compact_string());
  let mut syn = buffer_manager
    ._make_syntax_by_file_ext(&file_extension)
    .unwrap()
    .unwrap();

  let filename = tmpfile.path();
  let absolute_filename = filename.absolutize().unwrap();
  let file_content = std::fs::read_to_string(&absolute_filename).unwrap();

  let mut text_rope_builder: RopeBuilder = RopeBuilder::new();
  text_rope_builder.append(&file_content);
  let text_rope = text_rope_builder.finish();
  let syn_parser = syn.ts_parser();
  let (syn_tree, _editing_version, text_rope, text_payload) = syntax::_parse(
    syn_parser,
    None,
    vec![SyntaxEdit::New(SyntaxEditNew {
      payload: text_rope.clone(),
      version: 0,
    })],
  );
  let syn_capture = syntax::_query(
    &syn_tree,
    &text_rope,
    &text_payload,
    &syn.highlight_query(),
  );
  syn.set_highlight_capture(syn_capture);

  let colorscheme = lock!(cs_mgr).colorscheme().unwrap();
  (syn, colorscheme)
}
