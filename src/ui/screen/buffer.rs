use crate::ui::rect::Size;
use crate::ui::screen::cell::Cell;

pub struct Buffer {
  size: Size,
  cells: Vec<Cell>,
}

impl Buffer {
  pub fn new(size: Size) -> Self {
    Buffer {
      size,
      cells: vec![],
    }
  }

  pub fn size(&self) -> Size {
    self.size
  }

  pub fn height(&self) -> usize {
    self.size.height
  }

  pub fn width(&self) -> usize {
    self.size.width
  }
}
