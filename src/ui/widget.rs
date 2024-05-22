//! Basic atom of all UI components.

use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::ui::term::Terminal;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

pub mod root;

/// Widget is the base trait for all UI components, it provide a common layer for receiving user
/// events (keyboard/mouse), and rendering itself on terminal. It is more of a logical container
/// rather than a visible entity.
/// All widgets are maintained in a tree structure, i.e. the whole terminal is a root widget,
/// everything inside it is it's children widgets, and more grand-children widgets are nested
/// deeper inside these children of the root, and can recurse infinitely downwards.
/// The widget guarantee these parts:
/// 1. Children (include nested grand-children) will be destroyed when their parent is been
///    destroyed.
/// 2. Children (include nested grand-children) can only be **logically** placed outside of their
///    parent, while the outside parts will be invisible, i.e. only the parts inside their parent
///    geometric shape are visible.
/// 3. Each widget can bind an event handler, to handle the user events happening inside it & update
///    the content.
/// 4. The parent widget will be responsible for dispatching user events to the corresponding
///    child widget, based on whether user event is happening within the range of the widget
///    geometric shape.
/// 5. Children's attributes are by default inherited from their parent, if not explicitly set.
pub trait Widget {
  /// Unique ID of a widget instance.
  fn id(&self) -> usize;

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

  /// Parent widget.
  /// Note: root widget doesn't have a parent.
  fn parent(&self) -> Option<WidgetWk>;

  /// Children widgets.
  fn children(&self) -> LinkedList<WidgetWk>;

  /// Sibling widgets.
  fn siblings(&self) -> LinkedList<WidgetWk>;

  /// Draw the widget to terminal.
  fn draw(&self, t: &Terminal);
}

/// The `Rc/RefCell` smart pointer for a [widget](Widget).
pub type WidgetRc = Rc<RefCell<dyn Widget>>;
/// The `Weak/RefCell` smart pointer for a [widget](Widget).
pub type WidgetWk = Weak<RefCell<dyn Widget>>;
