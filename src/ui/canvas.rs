#![allow(dead_code)]

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, execute, terminal};
use std::io::stdout;
// use tracing::debug;
use crate::ui::rect::{Pos, Size};

pub async fn init() -> std::io::Result<Canvas> {
  terminal::enable_raw_mode()?;
  let (cols, rows) = terminal::size()?;
  let cvs = Canvas::new(Size::new(rows as u32, cols as u32), Pos::new(0, 0));

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
  size: Size,
  pos: Pos,
}

impl Canvas {
  fn new(size: Size, pos: Pos) -> Self {
    Canvas { size, pos }
  }

  fn height(&self) -> u32 {
    self.size.height
  }

  fn width(&self) -> u32 {
    self.size.width
  }

  fn x(&self) -> u32 {
    self.pos.x
  }

  fn y(&self) -> u32 {
    self.pos.y
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_canvas_new() {
    let c1 = Canvas::new(Size::new(1, 2), Pos::new(0, 0));
    assert_eq!(c1.height(), 1);
    assert_eq!(c1.width(), 2);
    assert_eq!(c1.x(), 0);
    assert_eq!(c1.y(), 0);
  }
}
