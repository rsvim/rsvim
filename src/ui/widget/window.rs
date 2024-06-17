//! The Vim window.

use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::ui::term::Terminal;
use crate::ui::widget::{ChildWidgetsRw, Widget, WidgetRw};
use crate::uuid;
use std::sync::{Arc, RwLock};

pub struct Window {
  id: usize,
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  zindex: usize,
  visible: bool,
  enabled: bool,
  parent: WidgetRw,
}

impl Window {
  pub fn new(size: Size, parent: WidgetRw) -> Self {
    Window {
      id: uuid::next(),
      offset: IPos::new(0, 0),
      abs_offset: UPos::new(0, 0),
      size,
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

  fn offset(&self) -> IPos {
    self.offset
  }

  fn set_offset(&mut self, _: IPos) {}

  fn abs_offset(&self) -> UPos {
    self.abs_offset
  }

  fn size(&self) -> Size {
    self.size
  }

  fn set_size(&mut self, value: Size) {
    self.size = value;
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

  fn parent(&self) -> Option<WidgetRw> {
    Some(self.parent.clone())
  }

  fn set_parent(&mut self, parent: Option<WidgetRw>) {
    if let Some(p) = parent {
      self.parent = p;
    }
  }

  fn children(&self) -> ChildWidgetsRw {
    Arc::new(RwLock::new(vec![]))
  }

  fn find_children(&self, _id: usize) -> Option<WidgetRw> {
    None
  }

  fn find_direct_children(&self, _id: usize) -> Option<WidgetRw> {
    None
  }

  fn draw(&self, _terminal: &Terminal) {
    todo!();
  }
}
