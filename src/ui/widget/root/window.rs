use crate::ui::frame::{Frame, FrameWk};
use crate::ui::geo::pos::{IPos, UPos};
use crate::ui::geo::size::Size;
use crate::ui::term::Terminal;
use std::collections::LinkedList;

pub struct RootWindow {
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  zindex: usize,
  parent: Option<FrameWk>,
}

impl Frame for RootWindow {
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

  fn parent(&self) -> Option<FrameWk> {
    self.parent.clone()
  }

  fn children(&self) -> LinkedList<FrameWk> {
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
    parent: Option<FrameWk>,
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
