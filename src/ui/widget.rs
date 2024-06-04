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
/// everything inside is its children widgets, and can nested recurse infinitely downwards.
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
/// 1. Parent owns its children.
///    * Children will be destroyed when their parent is.
///    * Children are displayed inside their parent's geometric shape, clipped by boundaries.
///    * The [visible](Widget::visible()) and [enabled](Widget::enabled()) attributes are
///      implicitly controlled by parent, unless they're explicitly been set.
/// 2. Children have higher priority to display and process input events than their parent.
///    * Parent will first try to dispatch user events to the corresponding child if the event
///    happens within the range of the child's geometric shape. If the child doesn't process the
///    event, then parent will try to process it.
///    * Children are always displayed on top of their parent. For children that shade each other,
///    the one with higher [z-index](Widget::zindex()) has higher priority to display and receive
///    events.
pub trait Widget {
  // { Attributes

  /// Get unique ID of a widget instance.
  fn id(&self) -> usize;

  /// Get (relative) offset based on parent widget.
  ///
  /// The anchor is always NW (North-West).
  fn offset(&self) -> IPos;

  /// Set (relative) offset.
  fn set_offset(&mut self, value: IPos);

  /// Get absolute offset based on whole [terminal](crate::ui::term::Terminal).
  ///
  /// The anchor is always NW (North-West).
  fn abs_offset(&self) -> UPos;

  /// Get size.
  fn size(&self) -> Size;

  /// Set size.
  fn set_size(&mut self, value: Size);

  /// Control arrange content stack when multiple children overlap on each other, a widget with
  /// higher z-index has higher priority to be displayed.
  ///
  /// Z-index only works for the children stack under the same parent, a child widget will
  /// always cover/override its parent. To change the visibility priority between children and
  /// parent, you need to directly set another parent for the children, or even switch the
  /// relationship between children and parent, i.e. make child the parent, make parent the child.
  fn zindex(&self) -> usize;

  /// Set z-index value.
  fn set_zindex(&mut self, value: usize);

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
  fn set_visible(&mut self, value: bool);

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
  fn set_enabled(&mut self, value: bool);

  // } Attributes

  // { Relationship

  /// Get parent.
  ///
  /// Root widget doesn't have a parent.
  fn parent(&self) -> Option<WidgetRw>;

  /// Set/change parent.
  fn set_parent(&mut self, parent: Option<WidgetRw>);

  /// Get children.
  fn children(&self) -> ChildWidgetsRw;

  /// Find child and offspring widget by ID.
  fn find_children(&self, id: usize) -> Option<WidgetRw>;

  /// Find direct child widget by ID, without offsprings.
  fn find_direct_children(&self, id: usize) -> Option<WidgetRw>;

  // } Relationship

  // { Contents

  /// Draw the widget to terminal.
  fn draw(&self, t: &Terminal);

  // } Contents
}

/// Rc/RefCell
pub type WidgetRc = Rc<RefCell<dyn Widget>>;

/// Weak/RefCell
pub type WidgetWk = Weak<RefCell<dyn Widget>>;

/// Arc/RwLock
pub type WidgetRw = RwLock<Arc<dyn Widget>>;

/// Arc/Mutex
pub type WidgetMt = Mutex<Arc<dyn Widget>>;

/// Rc/RefCell + LinkedList<Rc/RefCell>
pub type ChildWidgetsRc = Rc<RefCell<LinkedList<WidgetRc>>>;

/// Weak/RefCell + LinkedList<Rc/RefCell>
pub type ChildWidgetsWk = Weak<RefCell<LinkedList<WidgetRc>>>;

/// Arc/RwLock + LinkedList<Arc/RwLock>
pub type ChildWidgetsRw = RwLock<Arc<LinkedList<WidgetRw>>>;

/// Arc/Mutex + LinkedList<Arc/RwLock>
pub type ChildWidgetsMt = Mutex<Arc<LinkedList<WidgetRw>>>;
