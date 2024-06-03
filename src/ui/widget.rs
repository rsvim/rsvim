//! Basic atom of all UI components.

use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::ui::term::Terminal;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, RwLock};

pub mod root;

/// Widget is the base trait for all UI components, it provide a common layer for receiving user
/// events (keyboard/mouse), and rendering itself on terminal. It is more of a logical container
/// rather than a visible entity.
///
/// All widgets are maintained in a tree structure, i.e. the whole terminal is a root widget,
/// everything inside it is children widgets, and nested recurse infinitely downwards.
///
/// Here we have several terms:
///
/// * Parent: the parent widget node.
/// * Child: the child widget node.
/// * Ancestor: either the parent, or the parent of some ancestor of the node.
/// * Descendant: either the child, or the child of some descendant of the node.
/// * Sibling: nodes with the same parent.
///
/// The widget tree structure guarantees:
///
/// 1. Children will be destroyed when their parent is, and are also displayed inside their
///    parent's coordinate system, clipped by boundaries. Children are always on top of the canvas
///    layer over their parent. For those who overlap each other, the one with higher
///    [z-index](Widget::zindex()) has higher priority to display.
/// 2. Parent will dispatch user events to corresponding child if an event happens within the range
///    of the parent widget geometric shape. Each widget can bind handlers to process & update the
///    data.
/// 3. Children's attributes are by default inherited from their parent, if not explicitly set.
pub trait Widget {
  // { Life cycle

  /// Delete the widget itself (later), and remove it from parent.
  ///
  /// The widget cannot be just deleted right now, right here, due to life cycle management. For
  /// logic level, user cannot use it after deleted, for system level, the memory will be released
  /// after all references are removed.
  fn delete(&self);

  /// Create new widget based on the parent, with all default settings.
  fn new(parent: Option<WidgetWk>) -> dyn Widget;

  // } Life cycle

  // { Common attributes

  /// Get unique ID of a widget instance.
  fn id(&self) -> usize;

  /// Get (relative) offset based on parent widget.
  ///
  /// The anchor is always NW (North-West).
  fn offset(&self) -> IPos;

  /// Set (relative) offset.
  fn set_offset(&mut self, value: IPos) -> &mut Self;

  /// Get absolute offset based on whole [terminal](crate::ui::term::Terminal).
  ///
  /// The anchor is always NW (North-West).
  fn abs_offset(&self) -> UPos;

  /// Get size.
  fn size(&self) -> Size;

  /// Set size.
  fn set_size(&mut self, value: Size) -> &mut Self;

  /// Control arrange content stack when multiple children overlap on each other, a widget with
  /// higher z-index has higher priority to be displayed.
  ///
  /// Z-index only works for the children stack under the same parent, a child widget will
  /// always cover/override its parent. To change the visibility priority between children and
  /// parent, you need to directly set another parent for the children, or even switch the
  /// relationship between children and parent, i.e. make child the parent, make parent the child.
  fn zindex(&self) -> usize;

  /// Set z-index value.
  fn set_zindex(&mut self, value: usize) -> &mut Self;

  /// Whether the widget is visible.
  ///
  /// When invisible, user event will no longer been received or processed, and not rendered to
  /// terminal, just like it's deleted.
  fn visible(&self) -> bool;

  /// Make the widget visible/invisible.
  ///
  /// Hide a widget also implicitly hides all children and offsprings. Children or offsprings
  /// cannot be visible when parent is invisible.
  ///
  /// Show a widget also implicitly shows all children and offsprings, unless they have been
  /// explicitly made invisible.
  fn set_visible(&mut self, value: bool) -> &mut Self;

  /// Whether the widget is enabled.
  ///
  /// When disabled, user event will no longer been received or processed, but still visible, just
  /// like it's locked.
  fn enabled(&self) -> bool;

  /// Make the widget enabled/disabled.
  ///
  /// Disable a widget also implicitly disables all children and offsprings. Children or offsprings
  /// cannot be enabled when parent is disabled.
  ///
  /// Enable a widget also implicitly enables all children and offsprings, unless they have been
  /// explicitly disabled.
  fn set_enabled(&mut self, value: bool) -> &mut Self;

  // } Common attributes

  // { Relationship

  /// Get parent.
  ///
  /// Root widget doesn't have a parent.
  fn parent(&self) -> Option<WidgetWk>;

  /// Set/change parent.
  fn set_parent(&mut self, parent: Option<WidgetWk>) -> &mut Self;

  /// Get children.
  fn children(&self) -> LinkedList<WidgetWk>;

  /// Find child and offspring widget by ID.
  fn find_children(&self, id: usize) -> Option<WidgetWk>;

  /// Find direct child widget by ID, without offsprings.
  fn find_direct_children(&self, id: usize) -> Option<WidgetWk>;

  // } Relationship

  /// Draw the widget to terminal.
  fn draw(&self, t: &Terminal);
}

/// Reference pointer for a [widget](Widget).
pub type WidgetRc = Rc<RefCell<dyn Widget>>;

/// Weak pointer for a [widget](Widget).
pub type WidgetWk = Weak<RefCell<dyn Widget>>;

/// Read/write (shared) pointer for a [widget](Widget).
pub type WidgetRw = Arc<RwLock<dyn Widget>>;

/// Exclusive pointer for a [widget](Widget).
pub type WidgetEx = Arc<Mutex<dyn Widget>>;
