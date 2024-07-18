//! Basic atom of all UI components.

use std::any::Any;

use crate::cart::{IPos, IRect, USize};
use crate::geo_rect_as;
use crate::ui::tree::node::NodeId;
use crate::uuid;
use geo::{self, point};

pub mod cursor;
pub mod root;
pub mod window;

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
///
/// A widget's shape is always a rectangle, its position and size is stored by a `rect`, based on
/// its parent's shape. While rendering to the terminal device, we will need to calculate its
/// absolute position and actual size.
/// Based on the fact that widget's shape is often read, rarely modified, we use the copy-on-write
/// policy to avoid too many duplicated calculations. A widget calculates its absolute position and
/// actual size once it's relative position or logical size is been changed, and also caches the
/// result. Thus we simply get the cached results when need.
pub trait Widget: Any {
  /// Get unique ID of a widget instance.
  fn id(&self) -> NodeId;

  // Coordinates System {

  /// Get rect (relative position and logical size).
  fn rect(&self) -> IRect;

  /// Set rect.
  fn set_rect(&mut self, rect: IRect);

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
    let r2 = geo_rect_as!(r, usize);
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

  // Coordinates System }

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

  // Attributes {

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

  // Attributes }

  // Render {

  /// Draw the widget to terminal.
  fn draw(&mut self);

  // Render }
}

pub struct WidgetBase {
  id: NodeId,
  rect: IRect,
  zindex: usize,
  visible: bool,
  enabled: bool,
}

impl WidgetBase {
  pub fn new(rect: IRect, zindex: usize) -> Self {
    WidgetBase {
      id: uuid::next(),
      rect,
      zindex,
      visible: true,
      enabled: true,
    }
  }

  pub fn id(&self) -> NodeId {
    self.id
  }

  pub fn rect(&self) -> IRect {
    self.rect
  }

  pub fn set_rect(&mut self, rect: IRect) {
    self.rect = rect;
  }

  pub fn zindex(&self) -> usize {
    self.zindex
  }

  pub fn set_zindex(&mut self, value: usize) {
    self.zindex = value;
  }

  pub fn visible(&self) -> bool {
    self.visible
  }

  fn set_visible(&mut self, value: bool) {
    self.visible = value;
  }

  fn enabled(&self) -> bool {
    self.enabled
  }

  fn set_enabled(&mut self, value: bool) {
    self.enabled = value;
  }
}

#[macro_export]
macro_rules! define_widget_base_helpers {
  () => {
    fn id(&self) -> NodeId {
      self.base.id()
    }

    fn rect(&self) -> IRect {
      self.base.rect()
    }

    fn set_rect(&mut self, rect: IRect) {
      self.base.set_rect(rect);
    }

    fn zindex(&self) -> usize {
      self.base.zindex()
    }

    fn set_zindex(&mut self, zindex: usize) {
      self.base.set_zindex(zindex);
    }

    fn visible(&self) -> bool {
      self.base.visible()
    }

    fn set_visible(&mut self, value: bool) {
      self.base.set_visible(value);
    }

    fn enabled(&self) -> bool {
      self.base.enabled()
    }

    fn set_enabled(&mut self, value: bool) {
      self.base.set_enabled(value);
    }
  };
}
