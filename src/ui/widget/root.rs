//! Root View, this view will be initialized right after RSVIM start, unless it's running in
//! headless mode.

pub mod window;

use crate::ui::geo::pos::{IPos, UPos};
use crate::ui::geo::size::Size;
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetWk};
use std::collections::LinkedList;

pub struct RootWidget {
  pub size: Size,
}

impl Widget for RootWidget {
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

impl RootWidget {
  #[allow(dead_code)]
  /// Create new root view.
  ///
  /// * `terminal`: terminal.
  fn new(terminal: &Terminal) -> Self {
    RootWidget {
      size: terminal.size(),
    }
  }
}
