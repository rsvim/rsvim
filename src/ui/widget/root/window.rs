use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetWk};
use std::collections::LinkedList;

pub struct RootWindow {
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  zindex: usize,
  parent: Option<WidgetWk>,
}

impl Widget for RootWindow {
  fn offset(&self) -> IPos {
    self.offset
  }

  fn abs_offset(&self) -> UPos {
    self.abs_offset
  }

  fn size(&self) -> Size {
    self.size
  }

  fn zindex(&self) -> usize {
    self.zindex
  }

  fn parent(&self) -> Option<WidgetWk> {
    self.parent.clone()
  }

  fn children(&self) -> LinkedList<WidgetWk> {
    todo!();
  }

  fn draw(&self, _terminal: &Terminal) {
    todo!();
  }
}

impl RootWindow {
  #[allow(dead_code)]
  fn new(
    offset: IPos,
    abs_offset: UPos,
    size: Size,
    zindex: usize,
    parent: Option<WidgetWk>,
  ) -> Self {
    RootWindow {
      offset,
      abs_offset,
      size,
      zindex,
      parent,
    }
  }
}
