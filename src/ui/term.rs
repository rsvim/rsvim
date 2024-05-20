//! Backend terminal for receiving user inputs & canvas for UI rendering.

use crate::geo::size::Size;
use crate::ui::term::buffer::Buffer;
use crossterm::cursor;
use crossterm::event::{Event, KeyCode};
use tracing::debug;

pub mod buffer;
pub mod cell;

/// Backend terminal
pub struct Terminal {
  buf: Buffer,
  prev_buf: Buffer,
}

impl Terminal {
  pub fn new(size: Size) -> Self {
    Terminal {
      prev_buf: Buffer::new(size),
      buf: Buffer::new(size),
    }
  }

  pub fn size(&self) -> Size {
    self.buf.size
  }

  pub fn prev_size(&self) -> Size {
    self.prev_buf.size
  }

  /// Accept a terminal (keyboard/mouse) event.
  /// Returns `true` if continue event loop, `false` if quit.
  pub async fn accept(&mut self, event: Event) -> bool {
    println!("Event::{:?}\r", event);
    debug!("Event::{:?}", event);

    if event == Event::Key(KeyCode::Char('c').into()) {
      println!("Curosr position: {:?}\r", cursor::position());
    }

    // quit event loop
    if event == Event::Key(KeyCode::Esc.into()) {
      return false;
    }

    // continue event loop
    true
  }

  pub fn flush(&mut self) {
    self.prev_buf = self.buf.clone();
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
