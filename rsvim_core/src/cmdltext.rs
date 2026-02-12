//! Temporary contents except buffers.

use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::text::Text;
use crate::prelude::*;
use ropey::Rope;
use std::fmt::Debug;

/// Temporary contents except buffers.
pub struct CmdlineText {
  // Cmdline input text
  input: Text,

  // Cmdline message text
  message: Text,

  // Cmdline message history
  message_history: RingBuffer<String>,
}

arc_mutex_ptr!(CmdlineText);

impl CmdlineText {
  pub fn new(canvas_size: U16Size) -> Self {
    let cmdline_opts = BufferOptionsBuilder::default().build().unwrap();
    Self {
      input: Text::new(cmdline_opts, canvas_size, Rope::new()),
      message: Text::new(cmdline_opts, canvas_size, Rope::new()),
      message_history: RingBuffer::new(500),
    }
  }

  /// Command-line input content
  pub fn cmdline_input(&self) -> &Text {
    &self.input
  }

  /// Mutable command-line input content
  pub fn cmdline_input_mut(&mut self) -> &mut Text {
    &mut self.input
  }

  /// Command-line message
  pub fn cmdline_message(&self) -> &Text {
    &self.message
  }

  /// Mutable command-line message
  pub fn cmdline_message_mut(&mut self) -> &mut Text {
    &mut self.message
  }

  /// Command-line message history
  pub fn cmdline_message_history(&self) -> &RingBuffer<String> {
    &self.message_history
  }

  /// Mutable command-line message history
  pub fn cmdline_message_history_mut(&mut self) -> &mut RingBuffer<String> {
    &mut self.message_history
  }
}

impl Debug for CmdlineText {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("TextContents")
  }
}
