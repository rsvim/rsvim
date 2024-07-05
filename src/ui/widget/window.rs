//! The Vim window.

use crate::geo::{IRect, URect};
use crate::ui::term::Terminal;
use crate::ui::widget::{ChildWidgetsArc, Widget, WidgetArc};
use crate::uuid;
use geo::coord;
use std::sync::{Arc, RwLock};

/// The Vim window.
pub struct Window {
  parent: WidgetArc,
  id: usize,
  rect: IRect,
  abs_rect: URect,
  zindex: usize,
  visible: bool,
  enabled: bool,
}

impl Window {
  pub fn new(rect: IRect, parent: WidgetArc) -> Self {
    Window {
      id: uuid::next(),
      rect,
      abs_rect: URect::new(coord! {x:0,y:0}, coord! {x:0,y:0}),
      zindex: uuid::next(),
      visible: true,
      enabled: true,
      parent: parent.clone(),
    }
  }
}

impl Widget for Window {
  fn id(&self) -> usize {
    self.id
  }

  fn rect(&self) -> IRect {
    self.rect
  }

  fn set_rect(&mut self, rect: IRect) {
    self.rect = rect;
  }

  fn abs_rect(&self) -> URect {
    self.abs_rect
  }

  fn set_abs_rect(&mut self, rect: URect) {
    self.abs_rect = rect;
  }

  fn zindex(&self) -> usize {
    self.zindex
  }

  fn set_zindex(&mut self, value: usize) {
    self.zindex = value;
  }

  fn visible(&self) -> bool {
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

  fn parent(&self) -> Option<WidgetArc> {
    Some(self.parent.clone())
  }

  fn set_parent(&mut self, parent: Option<WidgetArc>) {
    if let Some(p) = parent {
      self.parent = p;
    }
  }

  fn children(&self) -> Option<ChildWidgetsArc> {
    None
  }

  fn find_children(&self, _id: usize) -> Option<WidgetArc> {
    None
  }

  fn find_direct_children(&self, _id: usize) -> Option<WidgetArc> {
    None
  }

  fn draw(&self, _terminal: &mut Terminal) {
    todo!();
  }
}
