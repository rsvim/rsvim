//! The `/dev/null` mode writer.

use crate::evloop::writer::StdoutWritable;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::Shader;

#[derive(Debug)]
/// The `/dev/null` mode writer, it eats everything and print nothing.
pub struct DevNullWriter {
  shaders: Vec<Shader>,
}

impl DevNullWriter {
  pub fn new() -> Self {
    Self { shaders: vec![] }
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

  fn write(&mut self, canvas: &mut Canvas) -> IoResult<()> {
    let shader = canvas.shade();
    self.shaders.push(shader);
    Ok(())
  }
}

impl Default for DevNullWriter {
  fn default() -> Self {
    Self::new()
  }
}

impl DevNullWriter {
  pub fn shaders(&self) -> &Vec<Shader> {
    &self.shaders
  }
}
