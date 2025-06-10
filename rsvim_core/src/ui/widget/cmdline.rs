//! Vim cmdline.

use crate::buf::BufferWk;

#[derive(Debug, Clone)]
/// The Vim cmdline.
pub struct Cmdline {
  // Content buffer.
  _buffer: BufferWk,
}
