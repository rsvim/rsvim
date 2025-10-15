//! Temporary contents except buffers.

use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::text::Text;
use crate::prelude::*;
use ringbuf::HeapRb;
use ringbuf::traits::{Producer, RingBuffer};
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

  /// Append an message to "command-line-message" widget.
  ///
  /// Because if user calls the `Rsvim.cmd.echo` API in `.rsvim.js` configs
  /// before the editor TUI initialize, the UI tree is not created, and the
  /// "command-line-message" widget inside UI tree does not exist.
  ///
  /// Thus we will have to store the printed messages here, with a ring-buffer.
  /// If the messages are just too many, old messages will be thrown, only new
  /// messages are left.
  ///
  /// And all messages will be print once the editor TUI is initialized.
  pub fn append_command_line_history(&mut self, payload: String) {
    self.command_line_message_history.push_overwrite(payload);
  }
}

impl Debug for TextContents {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("TextContents")
  }
}
