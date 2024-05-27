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
/// everything inside it is children widgets, and nested recurse infinitely downwards.
/// The widget guarantee these parts:
/// 1. Children will be destroyed when their parent is, and are also displayed inside their parent's
///    coordinate system, clipped by boundaries.
/// 2. Children always cover their parent's display, for children who cover each other, higher
///    [zindex](Widget::zindex()) will cover others.
/// 2. Each widget can bind event handlers to handle the user events happening inside it & update
///    the content.
/// 3. The parent widget will be responsible for dispatching user events to the corresponding child
///    widget, based on whether user event is happening within the range of the widget geometric
///    shape.
/// 4. Children's attributes are by default inherited from their parent, if not explicitly set.
pub trait Widget {
  // {
  // Life Cycle

  /// Delete the widget itself (later), and remove it from parent.
  /// Note: The widget usually cannot be just deleted right now, right here, due to some life cycle
  /// management issue. For logic level, user should never use it once it's deleted, for system
  /// level, the memory will be released after all references on the smart pointer are removed.
  fn delete(&self);

  /// Create new widget based on the parent, with all default settings.
  fn new(parent: Option<WidgetWk>);

  // }

  // {
  // Life Cycle

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

  /// Change parent widget.
  fn set_parent(&mut self, parent: Option<WidgetWk>);

  /// Children widgets.
  fn children(&self) -> LinkedList<WidgetWk>;

  /// Children widgets.
  fn find_children(&self, id: usize) -> Option<WidgetWk>;

  /// Children widgets.
  fn find_direct_children(&self, id: usize) -> Option<WidgetWk>;

  /// Draw the widget to terminal.
  fn draw(&self, t: &Terminal);
}

/// The `Rc/RefCell` smart pointer for a [widget](Widget).
pub type WidgetRc = Rc<RefCell<dyn Widget>>;
/// The `Weak/RefCell` smart pointer for a [widget](Widget).
pub type WidgetWk = Weak<RefCell<dyn Widget>>;
