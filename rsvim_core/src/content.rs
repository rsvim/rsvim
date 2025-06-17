//! Text contents except buffers.

use crate::buf::{BufferLocalOptionsBuilder, Text};
#[allow(unused_imports)]
use crate::{arc_impl, lock};

use paste::paste;
use ropey::Rope;

#[derive(Debug)]
/// Text contents except buffers.
pub struct Contents {
  cmdline_content: Text,
}

arc_impl!(Contents);

impl Contents {
  pub fn new(canvas_height: u16) -> Self {
    let cmdline_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    Self {
      cmdline_content: Text::new(canvas_height, Rope::new(), cmdline_opts),
    }
  }

  pub fn cmdline_content(&self) -> &Text {
    &self.cmdline_content
  }

  pub fn cmdline_content_mut(&mut self) -> &mut Text {
    &mut self.cmdline_content
  }
}
