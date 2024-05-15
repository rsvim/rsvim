use crate::ui::rect::{IPos, Size, UPos};
use crate::ui::screen::Screen;
use crate::ui::view::{View, ViewWk};

pub struct RootWindow {
  size: Size,
}

impl View for RootWindow {
  fn offset(&self) -> IPos {
    IPos::new(0, 0)
  }

  fn abs_offset(&self) -> UPos {
    UPos::new(0, 0)
  }

  fn size(&self) -> Size {
    self.size
  }

  fn zindex(&self) -> usize {
    0
  }

  fn parent(&self) -> Option<ViewWk> {
    None
  }

  fn draw(&self, screen: &Screen) {
    todo!();
  }
}

impl RootWindow {
  fn new(screen: &Screen) -> Self {
    RootWindow {
      size: screen.size(),
    }
  }
}
