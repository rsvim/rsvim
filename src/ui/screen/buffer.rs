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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_buffer_new() {
    let sz = Size::new(1, 2);
    let c1 = Screen {
      prev_buf: Buffer::new(sz),
      buf: Buffer::new(sz),
    };
    assert_eq!(c1.height(), 1);
    assert_eq!(c1.width(), 2);
    assert_eq!(c1.prev_height(), 1);
    assert_eq!(c1.prev_width(), 2);
  }
}
