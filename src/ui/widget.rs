//! Basic atom of all UI components.

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

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
pub trait Widget {
  fn id(&self) -> WidgetId;

  /// Draw the widget to terminal, on the specific shape.
  fn draw(&mut self, _actual_shape: U16Rect, _terminal: TerminalArc) {
    // Do nothing.
  }
}

#[derive(Debug, Clone)]
pub enum WidgetValue {
  RootContainer(RootContainer),
  WindowContainer(WindowContainer),
  WindowContent(WindowContent),
  Cursor(Cursor),
}

impl InodeValue for WidgetValue {
  fn id(&self) -> InodeId {
    Widget::id(self)
  }
}

impl Widget for WidgetValue {
  fn id(&self) -> InodeId {
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
