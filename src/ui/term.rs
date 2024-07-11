//! Backend terminal for receiving user inputs & canvas for UI rendering.

use std::sync::{Arc, RwLock};

use crate::geom::U16Size;
use crate::ui::frame::{Cell, Cursor, Frame};
use crossterm::cursor as termcursor;
use crossterm::event::{Event, KeyCode};
use tracing::debug;

/// Backend terminal
pub struct Terminal {
  frame: Frame,
  prev_frame: Frame,
}

impl Terminal {
  pub fn new(size: U16Size, cursor: Cursor) -> Self {
    Terminal {
      prev_frame: Frame::new(size, cursor),
      frame: Frame::new(size, cursor),
    }
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

  /// Accept a terminal (keyboard/mouse) event.
  /// Returns `true` if continue event loop, `false` if quit.
  pub async fn accept(&mut self, event: Event) -> bool {
    println!("Event::{:?}\r", event);
    debug!("Event::{:?}", event);

    if event == Event::Key(KeyCode::Char('c').into()) {
      println!("Curosr position: {:?}\r", termcursor::position());
    }

    // quit loop
    if event == Event::Key(KeyCode::Esc.into()) {
      return false;
    }

    // continue loop
    true
  }

  pub fn flush(&mut self) {
    self.prev_frame = self.frame.clone();
  }
}

pub type TerminalArc = Arc<RwLock<Terminal>>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_terminal_new() {
    let s = U16Size::new(3, 4);
    let c = Cursor::default();
    let t = Terminal::new(s, c);
    assert_eq!(t.frame().size, t.prev_frame().size);
    assert_eq!(t.frame().cursor, t.prev_frame().cursor);
  }
}
