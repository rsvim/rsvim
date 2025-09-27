//! Temporary contents except buffers.

use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::text::Text;
use crate::prelude::*;
use ringbuf::HeapRb;
use ropey::Rope;
use std::fmt::Debug;

/// Temporary contents except buffers.
pub struct TextContents {
  command_line_input: Text,
  command_line_message: Text,
  command_line_message_history: HeapRb<String>,
}

arc_mutex_ptr!(TextContents);

impl TextContents {
  pub fn new(canvas_size: U16Size) -> Self {
    let command_line_opts = BufferOptionsBuilder::default().build().unwrap();
    Self {
      command_line_input: Text::new(
        command_line_opts,
        canvas_size,
        Rope::new(),
      ),
      command_line_message: Text::new(
        command_line_opts,
        canvas_size,
        Rope::new(),
      ),
      command_line_message_history: HeapRb::new(500),
    }
  }

  /// Get "command line" input content
  pub fn command_line_input(&self) -> &Text {
    &self.command_line_input
  }

  /// Get mutable "command line" input content
  pub fn command_line_input_mut(&mut self) -> &mut Text {
    &mut self.command_line_input
  }

  /// Get "command line" message
  pub fn command_line_message(&self) -> &Text {
    &self.command_line_message
  }

  /// Get mutable "command line" message
  pub fn command_line_message_mut(&mut self) -> &mut Text {
    &mut self.command_line_message
  }

  /// Get "command line" message history
  pub fn command_line_message_history(&self) -> &HeapRb<String> {
    &self.command_line_message_history
  }

  /// Get mutable "command line" message history
  pub fn command_line_message_history_mut(&mut self) -> &mut HeapRb<String> {
    &mut self.command_line_message_history
  }
}

impl Debug for TextContents {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("TextContents")
  }
}
