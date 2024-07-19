//! Basic atom of all UI components.

pub mod cursor;
pub mod root;
pub mod window;

use std::any::Any;

use crate::cart::{IPos, IRect, USize};
use crate::geo_rect_as;
use crate::ui::tree::node::NodeId;
use crate::uuid;
use geo::{self, point};

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
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
