//! Basic atom of all UI components.

pub mod cursor;
pub mod window;

// Re-export
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::window::Window;

use std::any::Any;

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::tree::node::NodeId;

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
///
/// There's a special widget: layout. It can arranges its children widgets layout, i.e. horizontal,
/// vertical, flexible, etc.
///
/// Note: The layout's [`draw`](Widget::draw()) method still works if it has some actual content to
/// render, while all its children widgets renderings will cover it.
pub trait Widget: Any {
  /// Get unique ID of a widget instance.
  fn id(&self) -> NodeId;

  /// Draw the widget to terminal, on the specific shape.
  fn draw(&mut self, _actual_shape: &U16Rect, _terminal: TerminalWk) {
    // Do nothing.
  }

  /// Get the layout policy (only when it's a layout widget).
  ///
  /// Returns a layout policy if it is a layout widget.
  /// Returns `None` it is not.
  fn layout(&self) -> Option<Layout> {
    None
  }
}

pub struct Fixed {}

/// The design of layout policy borrows the concepts from some CSS frameworks
/// ([Material UI](https://github.com/mui/material-ui),
/// [Quasar Framework](https://github.com/quasarframework/quasar)), i.e. it not only supports
/// fixed/flexible size layouts, but also the
/// [CSS Flexbox](https://css-tricks.com/snippets/css/a-guide-to-flexbox/) theory's
/// grid/row/column, space and alignment.
///
/// Layout policy presents how its children should be arranges by the widget tree. The algorithms
/// and implementations are done inside the widget tree.
pub enum Layout {}
