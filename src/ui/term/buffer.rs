use crate::geo::pos::UPos;
use crate::geo::size::Size;
use crate::ui::term::cell::Cell;

#[derive(Debug, Clone)]
/// Rendering buffer, all UI components will first write symbols/grapheme/characters to a buffer,
/// then flushed to terminal. Terminal will save the previous buffer after flushed, and use it to
/// diff with next buffer, to find out difference and only flush those changed/dirty parts to
/// backend device.
pub struct Buffer {
  pub size: Size,
  pub cells: Vec<Cell>,
}

impl Buffer {
  /// Make new buffer with [size](crate::geo::size::Size).
  pub fn new(size: Size) -> Self {
    Buffer {
      size,
      cells: vec![Cell::default(); size.area()],
    }
  }

  /// Get the cell.
  pub fn get_cell(&self, pos: UPos) -> &Cell {
    &self.cells[pos.x * pos.y]
  }

  /// Get the mutable cell.
  pub fn mut_get_cell(&mut self, pos: UPos) -> &mut Cell {
    &mut self.cells[pos.x * pos.y]
  }

  /// Set the cell.
  pub fn set_cell(&mut self, pos: UPos, cell: Cell) -> &mut Self {
    self.cells[pos.x * pos.y] = cell;
    self
  }

  /// Get n continuously cells, start from position.
  pub fn get_cells(&self, pos: UPos, n: usize) -> &[Cell] {
    let start_at = pos.x * pos.y;
    let end_at = start_at + n;
    &self.cells[start_at..end_at]
  }

  /// Get n continuously mutable cells, start from position.
  pub fn mut_get_cells(&mut self, pos: UPos, n: usize) -> &mut [Cell] {
    let start_at = pos.x * pos.y;
    let end_at = start_at + n;
    &mut self.cells[start_at..end_at]
  }

  /// Set n continuously cells, start from position.
  /// Returns n old cells.
  pub fn set_cells(&mut self, pos: UPos, cells: Vec<Cell>) -> &mut Self {
    let start_at = pos.x * pos.y;
    let end_at = start_at + cells.len();
    self.cells.splice(start_at..end_at, cells);
    self
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
