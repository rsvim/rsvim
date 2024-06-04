use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::id;
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetRw};
use std::collections::LinkedList;

pub struct RootWidget {
  id: usize,
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  visible: bool,
  enabled: bool,
}

impl RootWidget {
  pub fn new(size: Size) -> Self {
    RootWidget {
      id: id::next(),
      offset: IPos::new(0, 0),
      abs_offset: UPos::new(0, 0),
      size,
      visible: true,
      enabled: true,
    }
  }
}

impl Widget for RootWidget {
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
    0
  }

  fn set_zindex(&mut self, _: usize) {}

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
    None
  }

  fn set_parent(&mut self, _: Option<WidgetRw>) {
    unimplemented!();
  }

  fn children(&self) -> LinkedList<WidgetRw> {
    todo!();
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
