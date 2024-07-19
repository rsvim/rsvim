//! Basic atom of all UI components.

pub mod cursor;
pub mod root;
pub mod window;

use std::any::Any;

use crate::cart::IRect;
use crate::ui::tree::node::NodeId;
use crate::uuid;

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
pub trait Widget: Any {
  /// Get unique ID of a widget instance.
  fn id(&self) -> NodeId;

  /// Draw the widget to terminal.
  fn draw(&mut self);
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
