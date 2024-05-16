use crate::ui::geo::pos::UPos;
use crate::ui::geo::size::Size;
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
  /// Make new buffer with size.
  pub fn new(size: Size) -> Self {
    Buffer {
      size,
      cells: vec![Cell::default(); size.area()],
    }
  }

  /// Get single cell on position.
  pub fn get_cell(&self, pos: UPos) -> &Cell {
    &self.cells[pos.x * pos.y]
  }

  pub fn mut_get_cell(&mut self, pos: UPos) -> &mut Cell {
    &mut self.cells[pos.x * pos.y]
  }

  pub fn set_cell(&mut self, pos: UPos, cell: Cell) -> &mut Self {
    self.cells[pos.x * pos.y] = cell;
    self
  }

  pub fn get_cells(&self, pos: UPos, count: usize) -> &[Cell] {
    let start_at = pos.x * pos.y;
    let end_at = start_at + count;
    &self.cells[start_at..end_at]
  }

  pub fn mut_get_cells(&mut self, pos: UPos, count: usize) -> &mut [Cell] {
    let start_at = pos.x * pos.y;
    let end_at = start_at + count;
    &mut self.cells[start_at..end_at]
  }

  pub fn set_cells(&mut self, pos: UPos, cells: Vec<Cell>) -> Vec<Cell> {
    let start_at = pos.x * pos.y;
    let end_at = start_at + cells.len();
    self.cells.splice(start_at..end_at, cells).collect()
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
