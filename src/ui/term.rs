//! Backend terminal for receiving user inputs & canvas for UI rendering.

use crate::geo::size::Size;
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
  pub fn new(size: Size, cursor: Cursor) -> Self {
    Terminal {
      prev_frame: Frame::new(size, cursor),
      frame: Frame::new(size, cursor),
    }
  }

  pub fn frame(&self) -> &Frame {
    &self.frame
  }

  pub fn frame_mut(&mut self) -> &mut Frame {
    &mut self.frame
  }

  pub fn prev_frame(&self) -> &Frame {
    &self.prev_frame
  }

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_terminal_new() {
    let sz = Size::new(1, 2);
    let cs = Cursor::default();
    let t = Terminal::new(sz, cs);
    assert_eq!(t.frame().size, t.prev_frame().size);
    assert_eq!(t.frame().cursor, t.prev_frame().cursor);
  }
}
