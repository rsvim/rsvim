//! Backend terminal for receiving user inputs & canvas for UI rendering.

use crate::ui::geo::size::Size;
use crate::ui::term::buffer::Buffer;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, queue, terminal};
use std::io::{Result, Write};
// use tracing::debug;

pub mod buffer;
pub mod cell;

pub async fn init() -> Result<Terminal> {
  if !terminal::is_raw_mode_enabled()? {
    terminal::enable_raw_mode()?;
  }

  let (cols, rows) = terminal::size()?;
  let size = Size::new(rows as usize, cols as usize);
  let t = Terminal::new(size);

  let mut out = std::io::stdout();

  queue!(out, EnableMouseCapture)?;
  queue!(out, EnableFocusChange)?;

  queue!(
    out,
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBlock,
    cursor::MoveTo(0, 0),
    cursor::Show,
  )?;

  out.flush()?;

  Ok(t)
}

pub async fn shutdown() -> Result<()> {
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

/// Backend terminal
pub struct Terminal {
  buf: Buffer,
  prev_buf: Buffer,
}

impl Terminal {
  pub fn size(&self) -> Size {
    self.buf.size
  }

  pub fn prev_size(&self) -> Size {
    self.prev_buf.size
  }

  pub fn flush(&mut self) {
    self.prev_buf = self.buf.clone();
  }

  pub fn new(size: Size) -> Self {
    Terminal {
      prev_buf: Buffer::new(size),
      buf: Buffer::new(size),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_terminal_new() {
    let sz = Size::new(1, 2);
    let c1 = Terminal::new(sz);
    assert_eq!(c1.size(), c1.prev_size());
  }
}
