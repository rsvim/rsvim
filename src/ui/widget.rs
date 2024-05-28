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
/// All widgets are maintained in a tree structure, i.e. the whole terminal is a root widget,
/// everything inside it is children widgets, and nested recurse infinitely downwards.
/// The widget guarantee these parts:
/// 1. Children will be destroyed when their parent is, and are also displayed inside their parent's
///    coordinate system, clipped by boundaries.
/// 2. Children always cover their parent's display, for children who cover each other, higher
///    [z-index](Widget::zindex()) will cover others.
/// 2. Each widget can bind event handlers to handle the user events happening inside it & update
///    the content.
/// 3. The parent widget will be responsible for dispatching user events to the corresponding child
///    widget, based on whether user event is happening within the range of the widget geometric
///    shape.
/// 4. Children's attributes are by default inherited from their parent, if not explicitly set.
pub trait Widget {
  // { Life cycle

  /// Delete the widget itself (later), and remove it from parent.
  /// Note: The widget cannot be just deleted right now, right here, due to some life cycle
  /// management issue. For logic level, user should never use it after been deleted, for system
  /// level, the memory will be released after all references on the smart pointer are removed.
  fn delete(&self);

  /// Create new widget based on the parent, with all default settings.
  fn new(parent: Option<WidgetWk>);

  // } Life cycle

  // { Common attributes

  /// Get unique ID of a widget instance.
  fn id(&self) -> usize;

  /// Get (relative) offset based on parent widget.
  /// Note: The anchor is always NW (North-West).
  fn offset(&self) -> IPos;

  /// Set (relative) offset.
  fn set_offset(&mut self, value: IPos);

  /// Get absolute offset based on whole [terminal](crate::ui::term::Terminal).
  /// Note: The anchor is always north-west.
  fn abs_offset(&self) -> UPos;

  /// Get size.
  fn size(&self) -> Size;

  /// Control arrange content stack when multiple children conflict on each other.
  /// A widget that has a higher z-index will be put to the top of the parent's widget's stack,
  /// which has higher priority to be displayed.
  /// Note: z-index only works for the children stack under the same parent, a child widget will
  /// always cover/override its parent. To change the visibility priority between children and
  /// parent, you need to directly set another parent for the children, or even switch the
  /// relationship between children and parent, i.e. make child the parent, make parent the child.
  fn zindex(&self) -> usize;

  /// Set z-index value.
  fn set_zindex(&mut self, value: usize);

  /// Whether the widget is enabled. When a widget is disabled, user event will no longer been
  /// received or processed, just like it's been deleted.
  fn enabled(&self) -> bool;

  /// Enable a widget.
  fn enable(&mut self);

  /// Disable a widget.
  fn disable(&mut self);

  /// Whether the widget is visible. When a widget is invisible, user event will still be
  /// received and processed, and all logic will keep running, but it will not be rendered to
  /// terminal.
  fn visible(&self) -> bool;

  /// Make the widget visible.
  fn show(&mut self);

  /// Make the widget invisible.
  fn hide(&mut self);

  // } Common attributes

  // { Parent-child relationship

  /// Parent widget.
  /// Note: root widget doesn't have a parent.
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
  /// Parent widget will first try to dispatch the event to its children based on their geometric
  /// shape, i.e. if an event happens inside one of its child, the parent will first dispatch the
  /// event to that child.
  /// If there're multiple children can handle an event, the one who has a higher z-index value
  /// wins. If there's no children can handle the event, the parent will then try to handle it by
  /// itself.
  /// When the child returns `true`, the event is been handled, and the parent doesn't need to
  /// handle it. Note: if the child want its parent to also handle the event (again), it has to
  /// explicitly call the parent's `event` method.
  /// When the child returns `false`, the event is been ignored, thus the parent will then try to
  /// find the next child to handle it.
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
