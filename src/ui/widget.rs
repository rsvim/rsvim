//! Basic atom of all UI components.

use tracing::debug;

use crate::cart::U16Rect;
use crate::ui::canvas::Canvas;
use crate::ui::tree::internal::inode::{InodeId, InodeValue};

// Re-export
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::root::RootContainer;
pub use crate::ui::widget::window::Window;

pub mod cursor;
pub mod root;
pub mod window;

pub type WidgetId = usize;

/// Base trait for all UI widgets.
pub trait Widget {
  /// Draw the widget to canvas, on the specific shape.
  fn draw(&mut self, actual_shape: U16Rect, _canvas: &mut Canvas) {
    // Do nothing.
    debug!("draw, actual shape:{:?}", actual_shape);
  }
}
