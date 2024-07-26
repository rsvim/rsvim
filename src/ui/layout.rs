//! Layout is a logical container that manages all its nested children widgets, and arranges their
//! layout and shapes.
//!
//! Layout is a special tree node, it's also a tree-structure when implemented in the widget
//! [tree](crate::ui::tree::Tree).

pub mod root;

use crate::cart::U16Rect;
use crate::ui::term::TerminalWk;
use crate::ui::tree::node::NodeId;
use crate::ui::widget::Widget;

// Re-export
pub use crate::ui::layout::root::RootLayout;

/// Layout widget is a special widget that has no specific shape or content, but works as a logical
/// container for nested children widgets, and arrange their layout.
///
/// To be a container, it has to be both a node inside the widget [tree](crate::ui::tree::Tree) and
/// a sub-tree structure itself (to implement the layout/shape management).
pub trait Layout: Widget {
  fn id(&self) -> NodeId;

  fn draw(&mut self, _actual_shape: &U16Rect, _terminal: TerminalWk) {
    // Do nothing.
  }
}
