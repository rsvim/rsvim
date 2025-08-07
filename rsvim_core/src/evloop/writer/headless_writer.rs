//! Headless mode writer.

use crate::evloop::writer::StdoutWriter;
use crate::prelude::*;
use crate::ui::canvas::Canvas;

#[derive(Debug)]
/// Headless mode writer, it prints to terminal command line.
pub struct HeadlessWriter {}

impl StdoutWriter for HeadlessWriter {
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
