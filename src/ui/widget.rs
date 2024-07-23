//! Basic atom of all UI components.

pub mod cursor;
pub mod root;
pub mod window;

// Re-export
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::root::RootWidget;
pub use crate::ui::widget::window::Window;

use std::any::Any;

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::tree::node::NodeId;

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
pub trait Widget: Any {
  /// Get unique ID of a widget instance.
  fn id(&self) -> NodeId;

  /// Draw the widget to terminal, on the specific shape.
  fn draw(&mut self, _actual_shape: &U16Rect, _terminal: TerminalWk) {
    // Do nothing.
  }
}
