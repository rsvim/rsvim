//! Temporary contents except buffers.

use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::text::Text;
use crate::prelude::*;
use ropey::Rope;
use std::fmt::Debug;

/// Temporary contents except buffers.
pub struct TextContents {
  cmdline_input: Text,
  cmdline_message: Text,
  cmdline_message_history: RingBuffer<String>,
}

arc_mutex_ptr!(TextContents);

impl TextContents {
  pub fn new(canvas_size: U16Size) -> Self {
    let cmdline_opts = BufferOptionsBuilder::default().build().unwrap();
    Self {
      cmdline_input: Text::new(cmdline_opts, canvas_size, Rope::new()),
      cmdline_message: Text::new(cmdline_opts, canvas_size, Rope::new()),
      cmdline_message_history: RingBuffer::new(500),
    }
  }

  /// Command-line input content
  pub fn cmdline_input(&self) -> &Text {
    &self.cmdline_input
  }

  /// Mutable command-line input content
  pub fn cmdline_input_mut(&mut self) -> &mut Text {
    &mut self.cmdline_input
  }

  /// Command-line message
  pub fn cmdline_message(&self) -> &Text {
    &self.cmdline_message
  }

  /// Mutable command-line message
  pub fn cmdline_message_mut(&mut self) -> &mut Text {
    &mut self.cmdline_message
  }

  /// Command-line message history
  pub fn cmdline_message_history(&self) -> &RingBuffer<String> {
    &self.cmdline_message_history
  }

  /// Mutable command-line message history
  pub fn cmdline_message_history_mut(&mut self) -> &mut RingBuffer<String> {
    &mut self.cmdline_message_history
  }
}

impl Debug for TextContents {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("TextContents")
  }
}
