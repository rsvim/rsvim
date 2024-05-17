use crate::ui::frame::{Frame, FrameWk};
use crate::ui::geo::pos::{IPos, UPos};
use crate::ui::geo::size::Size;
use crate::ui::term::Terminal;
use std::collections::LinkedList;

pub struct RootFrame {
  pub size: Size,
}

impl Frame for RootFrame {
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

  fn parent(&self) -> Option<FrameWk> {
    None
  }

  fn children(&self) -> LinkedList<FrameWk> {
    todo!();
  }

  fn draw(&self, _terminal: &Terminal) {
    todo!();
  }
}

impl RootFrame {
  #[allow(dead_code)]
  /// Create new root view.
  ///
  /// * `terminal`: terminal.
  fn new(terminal: &Terminal) -> Self {
    RootFrame {
      size: terminal.size(),
    }
  }
}
