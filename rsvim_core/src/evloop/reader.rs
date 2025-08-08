//! STDIN readers for rsvim.

use crate::prelude::*;
use crate::ui::canvas::Canvas;

pub trait StdinReadable {
  /// Initialize STDOUT.
  fn init(&self) -> IoResult<()>;

  /// Initialize STDOUT complete.
  fn init_complete(&mut self, canvas: &mut Canvas) -> IoResult<()>;

  /// Shutdown STDOUT.
  fn shutdown(&self) -> IoResult<()>;

  /// Write logical UI to STDOUT.
  fn write(&mut self, canvas: &mut Canvas) -> IoResult<()>;
}
