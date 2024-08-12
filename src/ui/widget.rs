//! Basic atom of all UI components.

use tracing::debug;

use crate::cart::U16Rect;
use crate::ui::term::TerminalArc;
use crate::ui::tree::internal::inode::{InodeId, InodeValue};

// Re-export
pub use crate::ui::widget::container::root::RootContainer;
pub use crate::ui::widget::container::window::WindowContainer;
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::window::content::WindowContent;

pub mod container;
pub mod cursor;
pub mod window;

pub type WidgetId = usize;

/// Base trait for all UI widgets.
pub trait Widget {
  fn id(&self) -> WidgetId;

  /// Draw the widget to terminal, on the specific shape.
  fn draw(&mut self, actual_shape: U16Rect, _terminal: TerminalArc) {
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

  fn draw(&mut self, actual_shape: U16Rect, terminal: TerminalArc) {
    match self {
      WidgetValue::RootContainer(w) => w.draw(actual_shape, terminal),
      WidgetValue::WindowContainer(w) => w.draw(actual_shape, terminal),
      WidgetValue::WindowContent(w) => w.draw(actual_shape, terminal),
      WidgetValue::Cursor(w) => w.draw(actual_shape, terminal),
    }
  }
}
