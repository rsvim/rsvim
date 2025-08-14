//! The `/dev/null` mode writer, it eats everything.

use crate::evloop::writer::StdoutWritable;
use crate::prelude::*;
use crate::ui::canvas::Canvas;

#[derive(Debug)]
/// Headless mode writer, it prints to terminal command line.
pub struct DevNullWriter {}

impl DevNullWriter {
  pub fn new() -> Self {
    Self {}
  }
}

impl StdoutWritable for DevNullWriter {
  fn init(&self) -> IoResult<()> {
    Ok(())
  }

  fn init_complete(&mut self, _canvas: &mut Canvas) -> IoResult<()> {
    Ok(())
  }

  fn shutdown(&self) -> IoResult<()> {
    Ok(())
  }

  fn write(&mut self, _canvas: &mut Canvas) -> IoResult<()> {
    Ok(())
  }
}

impl Default for DevNullWriter {
  fn default() -> Self {
    Self::new()
  }
}
