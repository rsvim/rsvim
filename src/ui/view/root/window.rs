use crate::ui::geo::pos::{IPos, UPos};
use crate::ui::geo::size::Size;
use crate::ui::term::Terminal;
use crate::ui::view::{View, ViewWk};
use std::collections::LinkedList;

pub struct RootWindow {
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  zindex: usize,
  parent: Option<ViewWk>,
}

impl View for RootWindow {
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

  fn parent(&self) -> Option<ViewWk> {
    self.parent.clone()
  }

  fn children(&self) -> LinkedList<ViewWk> {
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
    parent: Option<ViewWk>,
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
