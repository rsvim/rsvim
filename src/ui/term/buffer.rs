use crate::ui::rect::Size;
use crate::ui::term::cell::Cell;

#[derive(Debug, Clone)]
/// Buffer for rendering UI components, they will first write symbols/grapheme/characters to this
/// buffer, then flushed to terminal. Terminal will save the buffer been flushed, and use it to
/// diff with next new buffer, find out difference and only flush those changed/dirty parts to
/// backend device.
///
/// * `size`: Buffer size.
/// * `cells`: Buffer cells.
pub struct Buffer {
  pub size: Size,
  pub cells: Vec<Cell>,
}

impl Buffer {
  pub fn new(size: Size) -> Self {
    Buffer {
      size,
      cells: vec![Cell::default(); size.area()],
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_equal_on_buffer_new() {
    let sz = Size::new(1, 2);
    let b = Buffer::new(sz);
    assert_eq!(b.size.height, 1);
    assert_eq!(b.size.width, 2);
    assert_eq!(b.size.area(), 2);
    assert_eq!(b.cells.len(), b.size.area());
    for c in b.cells.into_iter() {
      assert_eq!(c, Cell::default());
    }
  }
}
