use crate::ui::rect::Size;
use crate::ui::screen::cell::Cell;

/// Buffer for rendering UI components, they will first write symbols/grapheme/characters to
/// this buffer, then flushed to terminal screen. Terminal screen will save the buffer been flushed,
/// and use it to diff with next new buffer, find out the difference and reduce the bytes flushed to
/// backend terminal device, i.e. the crossterm library.
///
/// * `size`: Buffer size.
/// * `cells`: Buffer cells.
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
    let b = Buffer::new(sz);
    assert_eq!(b.height(), 1);
    assert_eq!(b.width(), 2);
  }
}
