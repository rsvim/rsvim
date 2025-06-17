//! Text contents except buffers.

use crate::buf::{BufferLocalOptionsBuilder, Text};
#[allow(unused_imports)]
use crate::{arc_impl, lock};

use paste::paste;
use ropey::Rope;

#[derive(Debug)]
/// Text contents except buffers.
pub struct Contents {
  cmdline_text: Text,
}

arc_impl!(Contents);

impl Contents {
  pub fn new(canvas_height: u16) -> Self {
    let cmdline_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    Self {
      cmdline_text: Text::new(canvas_height, Rope::new(), cmdline_opts),
    }
  }
}
