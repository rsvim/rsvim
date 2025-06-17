//! Temporary contents except buffers.

use crate::arc_impl;
use crate::buf::{BufferLocalOptionsBuilder, Text};
#[allow(unused_imports)]
use crate::prelude::*;

use paste::paste;
use ropey::Rope;

#[derive(Debug)]
/// Temporary contents except buffers.
pub struct TemporaryContents {
  cmdline_content: Text,
}

arc_impl!(TemporaryContents);

impl TemporaryContents {
  pub fn new(canvas_size: U16Size) -> Self {
    let cmdline_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    Self {
      cmdline_content: Text::new(cmdline_opts, canvas_size, Rope::new()),
    }
  }

  pub fn cmdline_content(&self) -> &Text {
    &self.cmdline_content
  }

  pub fn cmdline_content_mut(&mut self) -> &mut Text {
    &mut self.cmdline_content
  }
}
