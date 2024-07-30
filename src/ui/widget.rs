//! Basic atom of all UI components.

pub mod cursor;
pub mod layout;
pub mod window;

// Re-export
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::window::Window;

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::tree::node::NodeId;

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
pub trait Widget {
  /// Draw the widget to terminal, on the specific shape.
  fn draw(&mut self, _actual_shape: &U16Rect, _terminal: TerminalWk) {
    // Do nothing.
  }
}
