//! STDOUT writers for rsvim.
//!
//! Rsvim has several running modes:
//! - Editor mode: The TUI text file editor.
//! - Headless mode (not implemented): The logical text editor without TUI. In
//!   this mode, the STDIN reads from command line instead of terminal's
//!   keyboard/mouse events, STDOUT/STDERR write to terminal instead of
//!   rendering TUI. Without TUI, the editing modes (normal, insert,
//!   command-line, visual, etc) is not useful any more, thus STDIN treats
//!   command line input as javascript scripts. And UI canvas no longer prints
//!   to STDOUT, instead, only message related APIs such as `console.log()`
//!   prints to STDOUT, which is similar to general purpose javascript-based
//!   runtime such as node/deno.

use crate::prelude::*;
use crate::ui::canvas::Canvas;
use dev_null_writer::DevNullWriter;
use editor_writer::EditorWriter;
use headless_writer::HeadlessWriter;

pub mod dev_null_writer;
pub mod editor_writer;
pub mod headless_writer;
mod tui;

pub trait StdoutWritable {
  /// Initialize STDOUT.
  fn init(&self) -> IoResult<()>;

  /// Initialize STDOUT complete.
  fn init_complete(&mut self, canvas: &mut Canvas) -> IoResult<()>;

  /// Shutdown STDOUT.
  fn shutdown(&self) -> IoResult<()>;

  /// Write logical UI to STDOUT.
  fn write(&mut self, canvas: &mut Canvas) -> IoResult<()>;
}

#[derive(Debug)]
/// The value holder for writer.
pub enum StdoutWriterValue {
  EditorWriter(EditorWriter),
  HeadlessWriter(HeadlessWriter),
  DevNullWriter(DevNullWriter),
}

impl StdoutWritable for StdoutWriterValue {
  fn init(&self) -> IoResult<()> {
    match self {
      StdoutWriterValue::EditorWriter(w) => w.init(),
      StdoutWriterValue::HeadlessWriter(w) => w.init(),
      StdoutWriterValue::DevNullWriter(w) => w.init(),
    }
  }

  fn init_complete(&mut self, canvas: &mut Canvas) -> IoResult<()> {
    match self {
      StdoutWriterValue::EditorWriter(w) => w.init_complete(canvas),
      StdoutWriterValue::HeadlessWriter(w) => w.init_complete(canvas),
      StdoutWriterValue::DevNullWriter(w) => w.init_complete(canvas),
    }
  }

  fn shutdown(&self) -> IoResult<()> {
    match self {
      StdoutWriterValue::EditorWriter(w) => w.shutdown(),
      StdoutWriterValue::HeadlessWriter(w) => w.shutdown(),
      StdoutWriterValue::DevNullWriter(w) => w.shutdown(),
    }
  }

  /// Write logical UI to STDOUT.
  fn write(&mut self, canvas: &mut Canvas) -> IoResult<()> {
    match self {
      StdoutWriterValue::EditorWriter(w) => w.write(canvas),
      StdoutWriterValue::HeadlessWriter(w) => w.write(canvas),
      StdoutWriterValue::DevNullWriter(w) => w.write(canvas),
    }
  }
}

impl StdoutWriterValue {
  pub fn editor() -> Self {
    StdoutWriterValue::EditorWriter(EditorWriter::new())
  }

  pub fn headless() -> Self {
    StdoutWriterValue::HeadlessWriter(HeadlessWriter::new())
  }

  pub fn dev_null() -> Self {
    StdoutWriterValue::DevNullWriter(DevNullWriter::new())
  }
}
