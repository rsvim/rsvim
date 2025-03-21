//! Basic atom of all UI components.

use crate::ui::canvas::Canvas;

// use tracing::trace;

pub mod cursor;
pub mod root;
pub mod window;

/// Base trait for all UI widgets.
pub trait Widgetable {
  /// Draw the widget to canvas, on the specific shape.
  fn draw(&self, _canvas: &mut Canvas) {
    // Do nothing.
    // trace!("draw canvas");
  }
}
