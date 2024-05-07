#![allow(dead_code)]

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, execute, terminal};
use std::io::stdout;
// use tracing::debug;

pub async fn init() -> std::io::Result<Canvas> {
  terminal::enable_raw_mode()?;
  let (cols, rows) = terminal::size()?;
  let cvs = Canvas::new(rows, cols);

  execute!(std::io::stdout(), EnableMouseCapture)?;
  execute!(std::io::stdout(), EnableFocusChange)?;

  execute!(
    stdout(),
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBlock,
    cursor::Show,
    cursor::MoveTo(0, 0),
  )?;

  Ok(cvs)
}

pub async fn shutdown() -> std::io::Result<()> {
  execute!(
    stdout(),
    DisableMouseCapture,
    DisableFocusChange,
    terminal::LeaveAlternateScreen,
  )?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  Ok(())
}

pub struct Canvas {
  // canvas height
  height: u16,
  // canvas width
  width: u16,
  // x coordinate of top-left corner
  x: u16,
  // y coordinate of top-left corner
  y: u16,
}

impl Canvas {
  fn new(height: u16, width: u16, x: u16, y: u16) -> Self {
    Canvas {
      height,
      width,
      x,
      y,
    }
  }

  fn height(&self) -> u16 {
    self.height
  }

  fn width(&self) -> u16 {
    self.width
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_canvas_new() {
    let c1 = Canvas::new(1, 2);
    assert_eq!(c1.height, 1);
    assert_eq!(c1.width, 2);
    assert_eq!(c1.height(), 1);
    assert_eq!(c1.width(), 2);
  }
}
