//! Basic atom of all UI components.

pub mod cursor;
pub mod root;
pub mod window;

use std::any::Any;

use crate::ui::tree::node::NodeId;

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
pub trait Widget: Any {
  /// Get unique ID of a widget instance.
  fn id(&self) -> NodeId;

  /// Draw the widget to terminal.
  fn draw(&mut self);
}
