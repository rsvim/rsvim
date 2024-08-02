//! Basic atom of all UI components.

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::tree::internal::inode::InodeValue;
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
  fn draw(&mut self, _actual_shape: U16Rect, _terminal: TerminalWk) {
    // Do nothing.
  }
}

#[derive(Debug, Clone)]
pub enum WidgetEnum {
  RootContainer(RootContainer),
  WindowContainer(WindowContainer),
  WindowContent(WindowContent),
  Cursor(Cursor),
}

impl InodeValue for WidgetEnum {}

impl Widget for WidgetEnum {
  fn draw(&mut self, actual_shape: U16Rect, terminal: TerminalWk) {
    match self {
      WidgetEnum::RootContainer(widget) => widget.draw(actual_shape, terminal),
      WidgetEnum::WindowContainer(widget) => widget.draw(actual_shape, terminal),
      WidgetEnum::WindowContent(widget) => widget.draw(actual_shape, terminal),
      WidgetEnum::Cursor(widget) => widget.draw(actual_shape, terminal),
    }
  }
}
