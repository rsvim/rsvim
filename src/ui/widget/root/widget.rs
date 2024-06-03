use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetWk};
use std::collections::LinkedList;

pub struct RootWidget {
  pub size: Size,
}

impl Widget for RootWidget {
  fn delete(&self) {
    unimplemented!();
  }

  fn new(_: Option<WidgetWk>) {
    unimplemented!();
  }

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

  fn parent(&self) -> Option<WidgetWk> {
    None
  }

  fn children(&self) -> LinkedList<WidgetWk> {
    todo!();
  }

  fn draw(&self, _terminal: &Terminal) {
    todo!();
  }
}

impl RootWidget {
  pub fn new(size: Size) -> Self {
    RootWidget { size }
  }
}
