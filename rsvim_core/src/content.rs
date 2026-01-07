//! Temporary contents except buffers.

use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::text::Text;
use crate::prelude::*;
use ringbuf::HeapRb;
use ropey::Rope;
use std::fmt::Debug;

/// Temporary contents except buffers.
pub struct TextContents {
  cmdline_input: Text,
  cmdline_message: Text,
  cmdline_message_history: HeapRb<String>,
}

arc_mutex_ptr!(TextContents);

impl TextContents {
  pub fn new(canvas_size: U16Size) -> Self {
    let cmdline_opts = BufferOptionsBuilder::default().build().unwrap();
    Self {
      cmdline_input: Text::new(cmdline_opts, canvas_size, Rope::new()),
      cmdline_message: Text::new(cmdline_opts, canvas_size, Rope::new()),
      cmdline_message_history: HeapRb::new(500),
    }
  }

  /// Get "command-line" input content
  pub fn cmdline_input(&self) -> &Text {
    &self.cmdline_input
  }

  /// Get mutable "command-line" input content
  pub fn cmdline_input_mut(&mut self) -> &mut Text {
    &mut self.cmdline_input
  }

  /// Get "command line" message
  pub fn command_line_message(&self) -> &Text {
    &self.cmdline_message
  }

  /// Get mutable "command line" message
  pub fn command_line_message_mut(&mut self) -> &mut Text {
    &mut self.cmdline_message
  }

  /// Get "command line" message history
  pub fn command_line_message_history(&self) -> &HeapRb<String> {
    &self.cmdline_message_history
  }

  /// Get mutable "command line" message history
  pub fn command_line_message_history_mut(&mut self) -> &mut HeapRb<String> {
    &mut self.cmdline_message_history
  }
}

impl Debug for TextContents {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("TextContents")
  }
}
