//! Temporary contents except buffers.

use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::text::Text;
use crate::prelude::*;
use ropey::Rope;

#[derive_where::derive_where(Debug)]
#[derive(rsvim_macro::ArcMutexPtr)]
/// Temporary contents except buffers.
pub struct CmdlineText {
  #[derive_where(skip)]
  // Cmdline input text
  input: Text,

  #[derive_where(skip)]
  // Cmdline message text
  message: Text,

  #[derive_where(skip)]
  // Cmdline message history
  message_history: RingBuffer<String>,
}

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
  pub fn input(&self) -> &Text {
    &self.input
  }

  /// Mutable command-line input content
  pub fn input_mut(&mut self) -> &mut Text {
    &mut self.input
  }

  /// Command-line message
  pub fn message(&self) -> &Text {
    &self.message
  }

  /// Mutable command-line message
  pub fn message_mut(&mut self) -> &mut Text {
    &mut self.message
  }

  /// Command-line message history
  pub fn message_history(&self) -> &RingBuffer<String> {
    &self.message_history
  }

  /// Mutable command-line message history
  pub fn message_history_mut(&mut self) -> &mut RingBuffer<String> {
    &mut self.message_history
  }
}
