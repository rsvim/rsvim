//! Basic atom of all UI components.

use tracing::debug;

use crate::cart::U16Rect;
use crate::ui::canvas::Canvas;
use crate::ui::tree::internal::inode::{InodeId, InodeValue};

// Re-export
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::root::RootContainer;
pub use crate::ui::widget::window::content::WindowContent;
pub use crate::ui::widget::window::WindowContainer;

pub mod cursor;
pub mod root;
pub mod window;

pub type WidgetId = usize;

/// Base trait for all UI widgets.
pub trait Widget {
  fn id(&self) -> WidgetId;

  /// Draw the widget to canvas, on the specific shape.
  fn draw(&mut self, actual_shape: U16Rect, _canvas: &mut Canvas) {
    // Do nothing.
    debug!("draw, actual shape:{:?}", actual_shape);
  }
}

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum WidgetValue {
  RootContainer(RootContainer),
  WindowContainer(WindowContainer),
  WindowContent(WindowContent),
  Cursor(Cursor),
}

impl InodeValue for WidgetValue {
  /// Get widget tree node ID.
  fn id(&self) -> InodeId {
    Widget::id(self)
  }
}

impl Widget for WidgetValue {
  /// Get widget ID.
  fn id(&self) -> WidgetId {
    match self {
      WidgetValue::RootContainer(w) => w.id(),
      WidgetValue::WindowContainer(w) => w.id(),
      WidgetValue::WindowContent(w) => w.id(),
      WidgetValue::Cursor(w) => w.id(),
    }
  }

  /// Draw widget with (already calculated) actual shape, on the canvas.
  fn draw(&mut self, actual_shape: U16Rect, canvas: &mut Canvas) {
    match self {
      WidgetValue::RootContainer(w) => w.draw(actual_shape, canvas),
      WidgetValue::WindowContainer(w) => w.draw(actual_shape, canvas),
      WidgetValue::WindowContent(w) => w.draw(actual_shape, canvas),
      WidgetValue::Cursor(w) => w.draw(actual_shape, canvas),
    }
  }
}
