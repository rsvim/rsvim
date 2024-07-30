//! Basic atom of all UI components.

pub mod container;
pub mod cursor;
pub mod window;

// Re-export
pub use crate::ui::widget::container::root::RootContainer;
pub use crate::ui::widget::container::window::WindowContainer;
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::window::content::WindowContent;

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
pub trait Widget {
  /// Draw the widget to terminal, on the specific shape.
  fn draw(&mut self, _actual_shape: &U16Rect, _terminal: TerminalWk) {
    // Do nothing.
  }
}

pub enum WidgetImpl {
  RootContainer(RootContainer),
  WindowContainer(WindowContainer),
  WindowContent(WindowContent),
  Cursor(Cursor),
}
