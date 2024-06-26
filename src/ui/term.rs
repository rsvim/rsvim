//! Backend terminal for receiving user inputs & canvas for UI rendering.

use crate::geo::U16Rect;
use crate::ui::frame::{Cursor, Frame};
use crossterm::cursor as termcursor;
use crossterm::event::{Event, KeyCode};
use geo::coord;
use tracing::debug;

/// Backend terminal
pub struct Terminal {
  frame: Frame,
  prev_frame: Frame,
}

impl Terminal {
  pub fn new(rect: U16Rect, cursor: Cursor) -> Self {
    Terminal {
      prev_frame: Frame::new(rect, cursor),
      frame: Frame::new(rect, cursor),
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
    let r = U16Rect::new(coord! {x:1,y:2}, coord! { x: 3, y: 4 });
    let c = Cursor::default();
    let t = Terminal::new(r, c);
    assert_eq!(t.frame().rect, t.prev_frame().rect);
    assert_eq!(t.frame().cursor, t.prev_frame().cursor);
  }
}
