//! Temporary contents except buffers.

use crate::arc_mutex_impl;
use crate::buf::opt::BufferLocalOptionsBuilder;
use crate::buf::text::Text;
use crate::prelude::*;

use paste::paste;
use ropey::Rope;

#[derive(Debug)]
/// Temporary contents except buffers.
pub struct TextContents {
  command_line_content: Text,
}

arc_mutex_impl!(TextContents);

impl TextContents {
  pub fn new(canvas_size: U16Size) -> Self {
    let command_line_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    Self {
      command_line_content: Text::new(command_line_opts, canvas_size, Rope::new()),
    }
  }

  pub fn command_line_content(&self) -> &Text {
    &self.command_line_content
  }

  pub fn command_line_content_mut(&mut self) -> &mut Text {
    &mut self.command_line_content
  }
}
