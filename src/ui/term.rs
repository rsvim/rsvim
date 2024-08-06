//! Backend terminal for receiving user inputs & canvas for UI rendering.

use crossterm::{self, queue};
use parking_lot::Mutex;
use std::io::Result as IoResult;
use std::sync::Arc;

use crate::cart::U16Size;
use crate::ui::frame::{Cell, Cursor, Frame};

/// Backend terminal
#[derive(Debug, Clone)]
pub struct Terminal {
  frame: Frame,
  prev_frame: Frame,
}

pub type TerminalArc = Arc<Mutex<Terminal>>;

impl Terminal {
  pub fn new(size: U16Size) -> Self {
    Terminal {
      prev_frame: Frame::new(size, Cursor::default()),
      frame: Frame::new(size, Cursor::default()),
    }
  }

  pub fn to_arc(t: Terminal) -> TerminalArc {
    Arc::new(Mutex::new(t))
  }

  // Current frame {

  pub fn frame(&self) -> &Frame {
    &self.frame
  }

  pub fn frame_mut(&mut self) -> &mut Frame {
    &mut self.frame
  }

  pub fn size(&self) -> U16Size {
    self.frame.size
  }

  pub fn set_size(&mut self, size: U16Size) {
    self.frame.size = size;
  }

  pub fn cells(&self) -> &Vec<Cell> {
    &self.frame.cells
  }

  pub fn cells_mut(&mut self) -> &mut Vec<Cell> {
    &mut self.frame.cells
  }

  pub fn cursor(&self) -> &Cursor {
    &self.frame.cursor
  }

  pub fn cursor_mut(&mut self) -> &mut Cursor {
    &mut self.frame.cursor
  }

  // Current frame }

  // Previous frame {

  pub fn prev_frame(&self) -> &Frame {
    &self.prev_frame
  }

  pub fn prev_size(&self) -> U16Size {
    self.prev_frame.size
  }

  pub fn prev_cells(&self) -> &Vec<Cell> {
    &self.prev_frame.cells
  }

  pub fn prev_cursor(&self) -> &Cursor {
    &self.prev_frame.cursor
  }

  // Previous frame }

  pub async fn flush(&mut self) -> IoResult<()> {
    // Dump current frame to device, with a diff-algorithm to reduce the output.
    if self.frame.dirty_cursor {
      let mut out = std::io::stdout();

      let cursor = self.frame.cursor;
      if cursor.blinking {
        queue!(out, crossterm::cursor::EnableBlinking)?;
      } else {
        queue!(out, crossterm::cursor::DisableBlinking)?;
      }
      if cursor.hidden {
        queue!(out, crossterm::cursor::Hide)?;
      } else {
        queue!(out, crossterm::cursor::Show)?;
      }
      queue!(out, cursor.style)?;
      queue!(
        out,
        crossterm::cursor::MoveTo(cursor.pos.x(), cursor.pos.y())
      )?;
    }

    // Save current frame.
    self.prev_frame = self.frame.clone();
    // Reset the `dirty` fields.
    self.frame.reset_dirty();

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new1() {
    let t = Terminal::new(U16Size::new(3, 4));
    assert_eq!(t.frame().size, t.prev_frame().size);
    assert_eq!(t.frame().cursor, t.prev_frame().cursor);
  }
}
