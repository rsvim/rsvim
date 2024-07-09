//! Basic atom of all UI components.

use crate::geo::{IPos, IRect, U16Size, UPos, URect, USize};
use crate::ui::term::Terminal;
use geo::{coord, point, Coord, Rect};
use std::any::Any;
use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::vec::Vec;

pub mod cursor;
pub mod root;
pub mod window;

/// Concrete struct type that implements a Widget trait.
pub enum WidgetType {
  RootWidgetType,
  CursorType,
  WindowType,
}

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
///    * Coordinates system are by default relative. i.e. children's relative positions are based
///      on their parent's top-left corner, absolute positions are based on the terminal's top-left
///      corner.
///    * Children are displayed inside their parent's geometric shape, clipped by boundaries. The
///      size system are by default logically infinite. i.e. children's logic shape can be
///      logically infinite on the canvas of imagination, while the actual (i.e. visible) shape is
///      been truncated by parent's shape.
///    * The [visible](Widget::visible()) and [enabled](Widget::enabled()) attributes are
///      implicitly controlled by parent, unless they're explicitly been set.
/// 2. Children have higher priority to display and process input events than their parent.
///    * Parent will first try to dispatch user events to the corresponding child if the event
///    happens within the range of the child's geometric shape. If the child doesn't process the
///    event, then parent will try to process it.
///    * Children are always displayed on top of their parent.
///    * For children that shade each other, the one with higher [z-index](Widget::zindex()) has
///      higher priority to display and receive events.
pub trait Widget {
  // { Attributes

  /// Get unique ID of a widget instance.
  fn id(&self) -> usize;

  fn typeid(&self) -> WidgetType;

  /// Get (relative) position.
  fn pos(&self) -> IPos;

  /// Set (relative) position.
  fn set_pos(&mut self, pos: IPos);

  /// Get absolute position.
  fn absolute_pos(&self) -> UPos;

  /// Set absolute position.
  fn set_absolute_pos(&mut self, pos: UPos);

  /// Get (logic) size.
  fn size(&self) -> USize;

  /// Set (logic) size.
  fn set_size(&mut self, size: USize);

  /// Get actual size.
  fn actual_size(&self) -> USize;

  /// Set actual size.
  /// If the actual size is out of parent's shape, it will be automatically truncated.
  fn set_actual_size(&mut self, size: USize);

  /// Get (relative) rect.
  /// It indicates both positions and (logic) size.
  fn rect(&self) -> IRect;

  /// Set (relative) rect.
  fn set_rect(&mut self, rect: IRect);

  /// Get absolute rect.
  fn absolute_rect(&self) -> URect;

  /// Set absolute rect.
  fn set_absolute_rect(&mut self, rect: URect);

  /// Get (relative) rect with actual size.
  fn actual_rect(&self) -> IRect;

  /// Set (relative) rect with actual size.
  /// If the actual size is out of parent's shape, it will be automatically truncated.
  fn set_actual_rect(&mut self, rect: IRect);

  /// Get absolute rect with actual size.
  fn actual_absolute_rect(&self) -> URect;

  /// Set absolute rect with actual size.
  /// If the actual size is out of parent's shape, it will be automatically truncated.
  fn set_actual_absolute_rect(&mut self, rect: URect);

  /// Control arrange content stack when multiple children overlap on each other, a widget with
  /// higher z-index has higher priority to be displayed.
  ///
  /// Note:
  /// 1. The z-index only works for the children stack under the same parent, a child widget will
  ///    always cover/override its parent. To change the visibility priority between children and
  ///    parent, you need to directly set another parent for the children, or even swap the
  ///    children and the parent.
  /// 2. For two children with different z-index, say A with 100, B with 10. When B has a child C
  ///    with z-index 1000, even 1000 > 100 > 10, A still covers C because it's a child of B.
  ///
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
  fn parent(&self) -> Option<WidgetArc>;

  /// Set/change parent.
  fn set_parent(&mut self, parent: Option<WidgetArc>);

  /// Get children.
  fn children(&self) -> Option<WidgetsArc>;

  /// Set children.
  fn set_children(&mut self, children: Option<WidgetsArc>);

  /// Find child and offspring widget by ID.
  fn find_children(&self, id: usize) -> Option<WidgetArc>;

  /// Find direct child widget by ID, without offsprings.
  fn find_direct_children(&self, id: usize) -> Option<WidgetArc>;

  // } Relationship

  // { Contents

  /// Draw the widget to terminal.
  fn draw(&self, t: &mut Terminal);

  // } Contents

  // Helpers {

  /// Children and parent's relative/absolute position, logic/actual size calculation.

  /// Calculate absolute position based on (relative) position and parent's absolute position.
  /// Note: If the absolute position is outside of the terminal, it will be automatically bounded
  /// inside of the terminal's shape.
  /// For example:
  /// 1. If current widget's relative position is (-1, -1), parent's absolute position is (0, 0).
  ///    Then the current widget's absolute position is (0, 0).
  /// 2. If current widget's relative position is (10, 10), parent's actual size is (5, 5),
  ///    terminal's actual size is (5, 5). Then the current widget's absolution position is (5, 5).
  fn to_absolute_pos(&self, terminal_size: U16Size) -> UPos {
    let p1 = self.pos();
    match self.parent() {
      Some(parent) => {
        let p2 = parent.read().unwrap().absolute_pos();
        let p3: IPos = p1 + point!(x: p2.x() as isize, y: p2.y() as isize);
        let x = min(max(p3.x(), 0), terminal_size.width as isize) as usize;
        let y = min(max(p3.y(), 0), terminal_size.height as isize) as usize;
        point!(x: x, y: y) as UPos
      }
      _ => unreachable!("No parent to calculate absolute position"),
    }
  }

  /// Calculate (relative) position based on absolute position and parent's absolute position.
  fn to_pos(&self) -> IPos {
    let p1 = self.absolute_pos();
    match self.parent() {
      Some(parent) => {
        let p2 = parent.read().unwrap().absolute_pos();
        point!(x: p1.x() as isize, y: p1.y() as isize)
          - point!(x: p2.x() as isize, y: p2.y() as isize)
      }
      _ => unreachable!("No parent to calculate (relative) position"),
    }
  }

  /// Calculate actual size, based on (logic) size and parent's actual size.
  fn to_actual_size(&self) -> USize {
    let r1 = self.rect();
    match self.parent() {
      Some(parent) => {
        let s1 = parent.read().unwrap().actual_size();
        let top_left = r1.min();
        let bottom_right: Coord<isize> = coord! {x: min(top_left.x as isize + r1.height() as isize, s1.height as isize), y: min(top_left.y as isize + r1.width() as isize, s1.width as isize)};
        USize::new(
          (bottom_right.y - top_left.y) as usize,
          (bottom_right.x - top_left.y) as usize,
        )
      }
      _ => unreachable!("No parent to calculate actual size"),
    }
  }

  /// Calculate (relative) rect with actual size, based on (logic) size and parent's actual size.
  fn to_actual_rect(&self) -> URect {
    URect::new(point!(x:1, y:2), point!(x:4, y:5))
  }

  /// Calculate absolute rect with actual size, based on (logic) size and parent's actual size.
  fn to_actual_absolute_rect(&self) -> URect {
    URect::new(point!(x:1, y:2), point!(x:4, y:5))
  }

  // Helpers }
}

pub type WidgetRc = Rc<RefCell<dyn Widget>>;

pub type WidgetArc = Arc<RwLock<dyn Widget>>;

pub type WidgetsRc = Vec<WidgetRc>;

pub type WidgetsArc = Vec<WidgetArc>;

/// Define Widget Rc/Arc converters.
#[macro_export]
macro_rules! define_widget_helpers {
  () => {
    /// Define Widget Rc/Arc pointer converters.
    pub fn downcast_rc(w: WidgetRc) -> Rc<RefCell<Self>> {
      let as_any = |&w1| -> &dyn Any { w1 };
      let casted = as_any(w).downcast_ref::<Rc<RefCell<Self>>>();
      match casted {
        Some(result) => result,
        None => panic!("Failed to downcast WidgetRc to Rc<RefCell<Self>>"),
      }
    }

    pub fn downcast_arc(w: WidgetArc) -> Arc<RwLock<Self>> {
      let as_any = |&w1| -> &dyn Any { w1 };
      let casted = as_any(w).downcast_ref::<Rc<RefCell<Self>>>();
      match casted {
        Some(result) => result,
        None => panic!("Failed to downcast WidgetArc to Arc<RwLock<Self>>"),
      }
    }

    pub fn upcast_rc(w: Self) -> WidgetRc {
      Rc::new(RefCell::new(w)) as WidgetRc
    }

    pub fn upcast_arc(w: Self) -> WidgetArc {
      Arc::new(RwLock::new(w)) as WidgetArc
    }
  };
}
