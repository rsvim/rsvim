//! Basic atom of all UI components.

pub mod cursor;
pub mod root;
pub mod window;

use std::any::Any;

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::tree::node::NodeId;

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
pub trait Widget: Any {
  /// Get unique ID of a widget instance.
  fn id(&self) -> NodeId;

  /// Draw the widget to terminal, on the specific shape.
  fn draw(&mut self, actual_shape: &U16Rect, terminal: TerminalWk);
}
