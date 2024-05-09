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
}
