use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, queue, terminal};
use std::io::{Result as IoResult, Write};
// use tracing::debug;
use crate::ui::canvas::buffer::Buffer;
use crate::ui::rect::Size;

pub mod buffer;
pub mod cell;

pub async fn init() -> std::io::Result<Canvas> {
  terminal::enable_raw_mode()?;
  let (cols, rows) = terminal::size()?;
  let size = Size::new(rows as usize, cols as usize);
  let cvs = Canvas {
    size,
    prev_buf: Buffer::new(size),
    buf: Buffer::new(size),
  };

  let mut out = std::io::stdout();

  queue!(out, EnableMouseCapture)?;
  queue!(out, EnableFocusChange)?;

  queue!(
    out,
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBlock,
    cursor::Show,
    cursor::MoveTo(0, 0),
  )?;

  out.flush()?;

  Ok(cvs)
}

pub async fn shutdown() -> IoResult<()> {
  let mut out = std::io::stdout();
  queue!(
    out,
    DisableMouseCapture,
    DisableFocusChange,
    terminal::LeaveAlternateScreen,
  )?;

  out.flush()?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  Ok(())
}

pub struct Canvas {
  size: Size,
  prev_buf: Buffer,
  buf: Buffer,
}

impl Canvas {
  pub fn height(&self) -> usize {
    self.size.height
  }

  pub fn width(&self) -> usize {
    self.size.width
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_canvas_new() {
    let sz = Size::new(1, 2);
    let c1 = Canvas {
      size: sz,
      prev_buf: Buffer::new(sz),
      buf: Buffer::new(sz),
    };
    assert_eq!(c1.height(), 1);
    assert_eq!(c1.width(), 2);
  }
}
