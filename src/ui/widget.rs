//! Basic atom of all UI components.

use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::ui::term::Terminal;
use crossterm::event::{Event, EventStream, KeyCode};
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

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
/// * Parent: the direct upper side node in the tree structure.
/// * Child: the direct down side node in the tree structure.
/// * Ancestor: the indirect upper side node in the tree structure, i.e. parent of parent.
/// * Offspring: the indirect down side node in the tree structure, i.e. child of child.
///
/// The widget tree structure guarantee:
///
/// 1. Children will be destroyed when their parent is, and are also displayed inside their
///    parent's coordinate system, clipped by boundaries. Children are always on top of the canvas
///    layer over their parent. For those who overlap each other, higher [z-index](Widget::zindex())
///    wins.
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
  fn new(parent: Option<WidgetWk>);

  // } Life cycle

  // { Common attributes

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

  /// Control arrange content stack when multiple children conflict on each other.
  ///
  /// A widget that has a higher z-index will be put to the top of the parent's widget's stack,
  /// which has higher priority to be displayed.
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
  /// Make a widget invisible also implicitly invisibles all children and offsprings. Children or
  /// offsprings cannot be visible when parent is invisible.
  ///
  /// Make a widget visible also implicitly visibles all children and offsprings, unless they have
  /// been explicitly made invisible.
  fn set_visible(&mut self, value: bool);

  /// Whether the widget is enabled. When a widget is disabled, user event will no longer been
  /// received or processed, but it's still visible.
  fn enabled(&self) -> bool;

  /// Set the widget enabled/disabled.
  /// Disable a widget also disables all its children, include nested grand-children. It's not
  /// possible to enable a child while its parent is disabled.
  /// Enable a widget also enables all its children, include nested grand-children, unless they
  /// have been explicitly disabled.
  fn set_enabled(&mut self, value: bool);

  // } Common attributes

  // { Parent-child relationship

  /// Parent widget.
  ///
  /// Root widget doesn't have a parent.
  fn parent(&self) -> Option<WidgetWk>;

  /// Change parent widget.
  fn set_parent(&mut self, parent: Option<WidgetWk>);

  /// Children widgets.
  fn children(&self) -> LinkedList<WidgetWk>;

  /// Find children widgets by ID, include nested grand children.
  fn find_children(&self, id: usize) -> Option<WidgetWk>;

  /// Find direct children widgets by ID, without nested grand children.
  fn find_direct_children(&self, id: usize) -> Option<WidgetWk>;

  // } Parent-child relationship

  // { Event

  /// Process a user keyboard/mouse event.
  ///
  /// Parent widget will first try to dispatch the event to its children based on their geometric
  /// shape, i.e. if an event happens inside one of its child, the parent will first dispatch the
  /// event to that child.
  ///
  /// If there're multiple children can handle an event, the one who has a higher z-index value
  /// wins. If there's no children can handle the event, the parent will then try to handle it by
  /// itself.
  ///
  /// When the child returns `true`, the event is been handled, and the parent doesn't need to
  /// handle it. If the child want its parent to also handle the event (again), it has to
  /// explicitly call the parent's `event` method.
  ///
  /// When the child returns `false`, the event is been ignored, thus the parent will then try to
  /// find the next child to handle it.
  ///
  /// When a child is invisible, the event is been ignored, thus next child or the parent will
  /// handle it.
  /// When a child is disabled, the event is been ignored, but
  fn event(&mut self, event: Event) -> bool;

  // } Event

  // { Flush to terminal

  /// Draw the widget to terminal.
  fn draw(&self, t: &Terminal);

  // } Flush to terminal
}

/// The `Rc/RefCell` smart pointer for a [widget](Widget).
pub type WidgetRc = Rc<RefCell<dyn Widget>>;
/// The `Weak/RefCell` smart pointer for a [widget](Widget).
pub type WidgetWk = Weak<RefCell<dyn Widget>>;
