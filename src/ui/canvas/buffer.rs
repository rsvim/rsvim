use crate::ui::canvas::cell::Cell;
use crate::ui::rect::Size;

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
