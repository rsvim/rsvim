use crate::ui::rect::{IPos, Size, UPos};
use crate::ui::term::Terminal;
use crate::ui::view::{View, ViewWk};

pub struct Window {
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  zindex: usize,
  parent: Option<ViewWk>,
}

impl View for Window {
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

  fn draw(&self, terminal: &Terminal) {
    todo!();
  }
}
