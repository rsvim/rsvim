//! Basic atom of all UI components.

use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::ui::term::Terminal;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

pub mod root;

/// Widget is the base trait for all UI components, it provide a common layer for receiving user
/// inputs, keyboard/mouse events, and rendering itself on terminal.
/// It is more of a logical container rather than a visible entity.
pub trait Widget {
  /// (Relative) offset based on parent widget.
  /// Note: The anchor is always north-west.
  fn offset(&self) -> IPos;

  /// Absolute offset based on whole [terminal](crate::ui::term::Terminal).
  /// Note: The anchor is always north-west.
  fn abs_offset(&self) -> UPos;

  /// Widget size.
  fn size(&self) -> Size;

  /// Control arrange content layout when multiple children conflict on each other.
  /// A widget that has a higher zindex will cover/override the lower one.
  ///
  /// Note: zindex only works for the children has the same parent, a child widget will always
  /// cover/override its parent.
  fn zindex(&self) -> usize;

  /// Parent widget of this one.
  ///
  /// Note: Root widget doesn't have a parent.
  fn parent(&self) -> Option<WidgetWk>;

  /// Children widgets of this one.
  ///
  /// Note: A widget **owns** all its children, thus recursively **owns** all its nested
  /// grandchildren and so on, which means:
  /// 1. The (grand)children will be destroyed once their parent is been destroyed.
  /// 2. The (grand)children can only be *logically* placed outside of their parent, but the outside
  ///    parts will be invisible, only those parts inside their parent's rectangle are visible.
  /// 3. Some attributes are by default inherited from their parent, if not explicitly set.
  fn children(&self) -> LinkedList<WidgetWk>;

  /// Draw the widget to terminal.
  fn draw(&self, terminal: &Terminal);
}

/// The `Rc/RefCell` smart pointer for a [widget](Widget).
pub type WidgetRc = Rc<RefCell<dyn Widget>>;
/// The `Weak/RefCell` smart pointer for a [widget](Widget).
pub type WidgetWk = Weak<RefCell<dyn Widget>>;
