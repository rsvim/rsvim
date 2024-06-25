//! Cursor widget.

use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::ui::term::Terminal;
use crate::ui::widget::{ChildWidgetsRw, Widget, WidgetRw};
use crate::uuid;
use std::sync::{Arc, RwLock};

pub struct Cursor {
  parent: WidgetRw,
  id: usize,
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  visible: bool,
  enabled: bool,
}

impl Cursor {
  pub fn new(parent: WidgetRw, offset: IPos, size: Size) -> Self {
    Cursor {
      parent,
      id: uuid::next(),
      offset,
      abs_offset: UPos::default(),
      size,
      visible: true,
      enabled: true,
    }
  }
}

impl Widget for Cursor {
  fn id(&self) -> usize {
    self.id
  }

  fn offset(&self) -> IPos {
    self.offset
  }

  fn set_offset(&mut self, value: IPos) {
    self.offset = value;
  }

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
    0
  }

  fn set_zindex(&mut self, _zindex: usize) {}

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
    assert!(parent.is_some());
    match parent {
      Some(p) => self.parent = p,
      _ => unreachable!(),
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

  fn draw(&self, _: &Terminal) {
    todo!();
  }
}
