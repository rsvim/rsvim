use crate::ui::rect::Size;
use crate::ui::screen::buffer::Buffer;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, queue, terminal};
use std::io::{Result as IoResult, Write};
// use tracing::debug;

pub mod buffer;
pub mod cell;

pub async fn init() -> std::io::Result<Screen> {
  terminal::enable_raw_mode()?;
  let (cols, rows) = terminal::size()?;
  let size = Size::new(rows as usize, cols as usize);
  let cvs = Screen {
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

pub struct Screen {
  prev_buf: Buffer,
  buf: Buffer,
}

impl Screen {
  pub fn size(&self) -> Size {
    self.buf.size
  }

  pub fn prev_size(&self) -> Size {
    self.prev_buf.size
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_screen_new() {
    let sz = Size::new(1, 2);
    let c1 = Screen {
      prev_buf: Buffer::new(sz),
      buf: Buffer::new(sz),
    };
    assert_eq!(c1.size().height, 1);
    assert_eq!(c1.size().width, 2);
    assert_eq!(c1.prev_size().height, 1);
    assert_eq!(c1.prev_size().width, 2);
  }
}
