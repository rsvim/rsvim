use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::id;
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetWk};
use std::collections::LinkedList;

pub struct RootWidget {
  id: usize,
  offset: IPos,
  size: Size,
  visible: bool,
  enabled: bool,
}

impl Widget for RootWidget {
  fn delete(&self) {
    unimplemented!();
  }

  fn new(_: Option<WidgetWk>) -> dyn Widget {
    unimplemented!();
  }

  fn id(&self) -> usize {
    self.id
  }

  fn offset(&self) -> IPos {
    IPos::new(0, 0)
  }

  fn set_offset(&mut self, value: IPos) -> &mut Self {
    self.offset = value;
    self
  }

  fn abs_offset(&self) -> UPos {
    UPos::new(0, 0)
  }

  fn size(&self) -> Size {
    self.size
  }

  fn set_size(&mut self, value: Size) -> &mut Self {
    self.size = value;
    self
  }

  fn zindex(&self) -> usize {
    0
  }

  fn set_zindex(&mut self, _: usize) -> &mut Self {
    self
  }

  fn visible(&self) -> bool {
    self.visible
  }

  fn set_visible(&mut self, value: bool) -> &mut Self {
    self.visible = value;
    self
  }

  fn enabled(&self) -> bool {
    self.enabled
  }

  fn set_enabled(&mut self, value: bool) -> &mut Self {
    self.enabled = value;
    self
  }

  fn parent(&self) -> Option<WidgetWk> {
    None
  }

  fn set_parent(&mut self, _: Option<WidgetWk>) -> &mut Self {
    self
  }

  fn children(&self) -> LinkedList<WidgetWk> {
    todo!();
  }

  fn find_children(&self, id: usize) -> Option<WidgetWk> {
    None
  }

  fn find_direct_children(&self, id: usize) -> Option<WidgetWk> {
    None
  }

  fn draw(&self, _terminal: &Terminal) {
    todo!();
  }
}

impl RootWidget {
  pub fn new(size: Size) -> Self {
    RootWidget {
      id: id::next(),
      offset: IPos::new(0, 0),
      size,
      visible: true,
      enabled: true,
    }
  }
}
