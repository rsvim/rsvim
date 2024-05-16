use crate::ui::geo::pos::{IPos, UPos};
use crate::ui::geo::size::Size;
use crate::ui::term::Terminal;
use crate::ui::view::{View, ViewWk};
use std::collections::LinkedList;

pub struct RootView {
  pub size: Size,
}

impl View for RootView {
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

  fn children(&self) -> LinkedList<ViewWk> {
    todo!();
  }

  fn draw(&self, _terminal: &Terminal) {
    todo!();
  }
}

impl RootView {
  #[allow(dead_code)]
  /// Create new root view.
  ///
  /// * `terminal`: terminal.
  fn new(terminal: &Terminal) -> Self {
    RootView {
      size: terminal.size(),
    }
  }
}
