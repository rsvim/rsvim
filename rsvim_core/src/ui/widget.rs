//! Basic atom of all UI components.

#![allow(unused_imports, dead_code)]

use tracing::debug;

use crate::ui::canvas::Canvas;

// Re-export
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::root::RootContainer;
pub use crate::ui::widget::window::Window;

pub mod bufview;
pub mod cursor;
pub mod ptr;
pub mod root;
pub mod window;

/// Base trait for all UI widgets.
pub trait Widgetable {
  /// Draw the widget to canvas, on the specific shape.
  fn draw(&mut self, _canvas: &mut Canvas) {
    // Do nothing.
    // debug!("draw canvas");
  }
}
