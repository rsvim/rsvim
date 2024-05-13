use crate::ui::layout::LayoutRc;
use crate::ui::rect::{IPos, Size, UPos};
use crate::ui::screen::Screen;
use crate::ui::view::{View, ViewWk};
use std::rc::{Rc, Weak};
use tracing::debug;

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

  fn layout(&self) -> Option<LayoutRc> {
    None
  }

  fn draw(&self, screen: &Screen) {
    debug!("draw");
  }
}
