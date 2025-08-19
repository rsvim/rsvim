//! Temporary contents except buffers.

use crate::buf::opt::BufferLocalOptionsBuilder;
use crate::buf::text::Text;
use crate::prelude::*;

use ropey::Rope;

#[derive(Debug)]
/// Temporary contents except buffers.
pub struct TextContents {
  command_line_content: Text,
  command_line_message: Text,
}

arc_mutex_ptr!(TextContents);

impl TextContents {
  pub fn new(canvas_size: U16Size) -> Self {
    let command_line_opts =
      BufferLocalOptionsBuilder::default().build().unwrap();
    Self {
      command_line_content: Text::new(
        command_line_opts,
        canvas_size,
        Rope::new(),
      ),
      command_line_message: Text::new(
        command_line_opts,
        canvas_size,
        Rope::new(),
      ),
    }
  }

  /// Get "command line" input content
  pub fn command_line_content(&self) -> &Text {
    &self.command_line_content
  }

  /// Get mutable "command line" input content
  pub fn command_line_content_mut(&mut self) -> &mut Text {
    &mut self.command_line_content
  }

  /// Get "command line" echo message
  pub fn command_line_message(&self) -> &Text {
    &self.command_line_message
  }

  /// Get mutable "command line" echo message
  pub fn command_line_message_mut(&mut self) -> &mut Text {
    &mut self.command_line_message
  }
}
