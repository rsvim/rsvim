//! Basic atom of all UI components.

use crate::as_geo_rect;
use crate::geo::widget::relation;
use crate::geo::{IPos, IRect, U16Size, UPos, USize};
use crate::ui::term::Terminal;
use geo::{point, Rect};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::vec::Vec;

pub mod cursor;
pub mod root;
pub mod window;

/// Concrete struct kind (since `type` is a rust keyword that we need to avoid) that implements a
/// Widget trait.
pub enum WidgetKind {
  RootWidgetKind,
  CursorKind,
  WindowKind,
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
///      size system are by default logically infinite on parent's canvas of imagination, while the
///      actual (visible) shape is been truncated by parent's shape.
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

  fn kind(&self) -> WidgetKind;

  /// Get rect, relative position and logical
  fn rect(&self) -> IRect;

  /// Set rect.
  fn set_rect(&mut self, rect: IRect);

  // Position/Size/Rect Helpers {

  /// Get relative position.
  fn pos(&self) -> IPos {
    point!(x: self.rect().min().x, y: self.rect().min().y)
  }

  /// Set relative position.
  fn set_pos(&mut self, pos: IPos) {
    let r = self.rect();
    self.set_rect(IRect::new(
      pos,
      point!(x: pos.x() + r.width(), y: pos.y() + r.height()),
    ));
  }

  /// Get logical size.
  fn size(&self) -> USize {
    let r = self.rect();
    let r2 = as_geo_rect!(r, usize);
    USize::from(r2)
  }

  /// Set logical size.
  fn set_size(&mut self, sz: USize) {
    let r = self.rect();
    let bottom_left = r.min();
    self.set_rect(IRect::new(
      bottom_left.into(),
      point!(x: bottom_left.x + sz.width() as isize, y: bottom_left.y + sz.height() as isize),
    ));
  }

  /// Calculate absolute position, based on relative position and parent's absolute position.
  /// If there's no parent widget, the relative position itself is already absolute.
  fn get_absolute_pos(&self, terminal_size: U16Size) -> UPos {
    match self.parent() {
      Some(parent) => {
        let parent_absolute_pos = parent.read().unwrap().get_absolute_pos(terminal_size);
        relation::make_absolute_pos(self.pos(), Some(parent_absolute_pos), terminal_size)
      }
      None => relation::make_absolute_pos(self.pos(), None, terminal_size),
    }
  }

  // Position/Size/Rect Helpers }

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
