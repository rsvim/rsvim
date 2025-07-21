//! Internal implementations for `Iframe`.

use crate::prelude::*;
use crate::ui::canvas::frame::cell::Cell;

use geo::point;
use std::ops::Range;
// use tracing::trace;

#[derive(Debug, Clone)]
/// Internal implementation for `Iframe`.
pub struct Iframe {
  size: U16Size,
  cells: Vec<Cell>,

  /// Indicate what rows of the frame is dirty.
  ///
  /// NOTE: This is for fast locating the changed rows inside the terminal device, i.e. the whole
  /// TUI screen instead of the rows inside UI widget window.
  dirty_rows: Vec<bool>,
}

impl Iframe {
  /// Make new frame.
  pub fn new(size: U16Size) -> Self {
    let n = size.height() as usize * size.width() as usize;
    Iframe {
      size,
      cells: vec![Cell::default(); n],
      dirty_rows: vec![false; size.height() as usize], // When a frame first create, it's not dirty.
    }
  }

  // Utils {

  /// Convert start position and length of following N elements into Vec range.
  ///
  /// Returns the left-inclusive right-exclusive index range.
  pub fn pos2range(&self, pos: U16Pos, n: usize) -> Range<usize> {
    let start_idx = self.pos2idx(pos);
    let end_idx = start_idx + n;
    start_idx..end_idx
  }

  /// Convert start index and length of following N elements into Vec range.
  pub fn idx2range(&self, index: usize, n: usize) -> Range<usize> {
    let end_idx = index + n;
    index..end_idx
  }

  /// Convert (position) X and Y into Vec index.
  pub fn xy2idx(&self, x: usize, y: usize) -> usize {
    y * self.size.width() as usize + x
  }

  /// Convert position into Vec index.
  pub fn pos2idx(&self, pos: U16Pos) -> usize {
    self.xy2idx(pos.x() as usize, pos.y() as usize)
  }

  /// Convert index into (position) X and Y.
  ///
  /// Returns `(x, y)`.
  ///
  /// # Panics
  ///
  /// If index is outside of frame shape.
  pub fn idx2xy(&self, index: usize) -> (usize, usize) {
    debug_assert!(index <= self.cells.len());
    let x = index % self.size.width() as usize;
    let y = index / self.size.width() as usize;
    (x, y)
  }

  /// Convert index into position.
  ///
  /// # Panics
  ///
  /// If index is outside of frame shape.
  pub fn idx2pos(&self, index: usize) -> U16Pos {
    let (x, y) = self.idx2xy(index);
    point!(x: x as u16, y: y as u16)
  }

  // Utils }

  /// Get current frame size.
  pub fn size(&self) -> U16Size {
    self.size
  }

  /// Whether the frame is zero sized.
  pub fn zero_sized(&self) -> bool {
    self.size.height() == 0 || self.size.width() == 0
  }

  /// Set current frame size.
  pub fn set_size(&mut self, size: U16Size) -> U16Size {
    let old_size = self.size;
    self.size = size;
    self.cells.resize(
      size.height() as usize * size.width() as usize,
      Cell::default(),
    );
    self.dirty_rows = vec![true; size.height() as usize];
    old_size
  }

  /// Whether index is inside frame cells.
  pub fn contains_index(&self, index: usize) -> bool {
    index < self.cells.len()
  }

  /// Whether range is inside frame cells. The range is left-inclusive, right-exclusive.
  pub fn contains_range(&self, range: &Range<usize>) -> bool {
    range.start < self.cells.len() && range.end <= self.cells.len()
  }

  /// Get a cell.
  ///
  /// # Panics
  ///
  /// If the position is outside of frame shape.
  pub fn get_cell(&self, pos: U16Pos) -> &Cell {
    self.try_get_cell(pos).unwrap()
  }

  /// Try get a cell, non-panic version of [`get_cell`](Iframe::get_cell).
  pub fn try_get_cell(&self, pos: U16Pos) -> Option<&Cell> {
    let index = self.pos2idx(pos);
    if self.contains_index(index) {
      let result = &self.cells[index];
      // trace!(
      //   "try get cell at pos:{:?}, index:{:?}, cell:{:?}",
      //   pos,
      //   index,
      //   result
      // );
      Some(result)
    } else {
      // trace!("try get cell invalid at pos:{:?}, index:{:?}", pos, index);
      None
    }
  }

  /// Set a cell.
  ///
  /// Returns the old cell.
  ///
  /// # Panics
  ///
  /// If the position is outside of frame shape.
  pub fn set_cell(&mut self, pos: U16Pos, cell: Cell) -> Cell {
    self.try_set_cell(pos, cell).unwrap()
  }

  /// Try set a cell, non-panic version of [`set_cell`](Iframe::set_cell).
  pub fn try_set_cell(&mut self, pos: U16Pos, cell: Cell) -> Option<Cell> {
    let index = self.pos2idx(pos);
    if self.contains_index(index) {
      let old_cell = self.cells[index].clone();
      // trace!(
      //   "try set cell at index:{:?}, new cell:{:?}, old cell:{:?}",
      //   index,
      //   cell,
      //   old_cell
      // );
      self.cells[index] = cell;
      self.dirty_rows[pos.y() as usize] = true;
      Some(old_cell)
    } else {
      // trace!("try set cell invalid index:{:?}, cell:{:?}", index, cell);
      None
    }
  }

  /// Set an empty cell.
  ///
  /// Returns the old cell.
  ///
  /// # Panics
  ///
  /// If the position is outside of frame shape.
  pub fn set_empty_cell(&mut self, pos: U16Pos) -> Cell {
    self.set_cell(pos, Cell::empty())
  }

  /// Try set an empty cell, non-panic version of [`set_empty_cell`](Iframe::set_empty_cell).
  pub fn try_set_empty_cell(&mut self, pos: U16Pos) -> Option<Cell> {
    self.try_set_cell(pos, Cell::empty())
  }

  /// Get all cells.
  pub fn get_cells(&self) -> &Vec<Cell> {
    &self.cells
  }

  /// Get a range of continuously cells.
  ///
  /// # Panics
  ///
  /// If the range is outside of frame shape.
  pub fn get_cells_at(&self, pos: U16Pos, n: usize) -> &[Cell] {
    self.try_get_cells_at(pos, n).unwrap()
  }

  /// Try get a range of continuously cells, non-panic version of
  /// [`get_cells_at`](Iframe::get_cells_at).
  pub fn try_get_cells_at(&self, pos: U16Pos, n: usize) -> Option<&[Cell]> {
    let range = self.pos2range(pos, n);
    if self.contains_range(&range) {
      Some(&self.cells[range])
    } else {
      // trace!("try get cells at invalid range:{:?}", range);
      None
    }
  }

  /// Set (replace) cells at a range.
  ///
  /// Returns old cells.
  ///
  /// # Panics
  ///
  /// If any positions of `cells` is outside of frame shape.
  pub fn set_cells_at(&mut self, pos: U16Pos, cells: Vec<Cell>) -> Vec<Cell> {
    self.try_set_cells_at(pos, cells).unwrap()
  }

  /// Try set (replace) cells at a range, non-panic version of
  /// [`set_cells_at`](Iframe::set_cells_at).
  pub fn try_set_cells_at(
    &mut self,
    pos: U16Pos,
    cells: Vec<Cell>,
  ) -> Option<Vec<Cell>> {
    let range = self.pos2range(pos, cells.len());
    // trace!(
    //   "try set cells at range:{:?}, cells len:{:?}",
    //   range,
    //   self.cells.len()
    // );
    if self.contains_range(&range) {
      let end_at = self.idx2pos(range.end);
      // trace!("try set dirty rows for pos:{:?}, end_at:{:?}", pos, end_at);
      for row in pos.y()..(end_at.y() + 1) {
        // trace!("try set dirty rows at row:{:?}", row);
        if (row as usize) < self.dirty_rows.len() {
          self.dirty_rows[row as usize] = true;
        }
      }
      // trace!(
      //   "try set cells dirty at row range:{:?}-{:?}",
      //   pos.y(),
      //   end_at.y() + 1
      // );
      Some(self.cells.splice(range, cells).collect())
    } else {
      None
    }
  }

  /// Set (replace) empty cells at a range.
  ///
  /// # Panics
  ///
  /// If any positions of `cells` is outside of frame shape.
  pub fn set_empty_cells_at(&mut self, pos: U16Pos, n: usize) -> Vec<Cell> {
    self.set_cells_at(pos, vec![Cell::empty(); n])
  }

  /// Try set (replace) empty cells at a range, non-panic version of
  /// [`set_empty_cells_at`](Iframe::set_empty_cells_at).
  pub fn try_set_empty_cells_at(
    &mut self,
    pos: U16Pos,
    n: usize,
  ) -> Option<Vec<Cell>> {
    self.try_set_cells_at(pos, vec![Cell::empty(); n])
  }

  /// Get dirty rows.
  pub fn dirty_rows(&self) -> &Vec<bool> {
    &self.dirty_rows
  }

  /// Reset/clean all dirty components.
  ///
  /// NOTE: This method should be called after current frame flushed to terminal device.
  pub fn reset_dirty_rows(&mut self) {
    self.dirty_rows = vec![false; self.size.height() as usize];
  }
}

#[cfg(test)]
use compact_str::CompactString;

impl Iframe {
  #[cfg(test)]
  /// Get raw symbols of all cells.
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols(&self) -> Vec<Vec<CompactString>> {
    let mut results: Vec<Vec<CompactString>> = vec![];
    for row in 0..self.size.height() {
      let mut row_symbols: Vec<CompactString> = vec![];
      for col in 0..self.size.width() {
        let idx = self.xy2idx(col as usize, row as usize);
        row_symbols.push(self.cells[idx].symbol().clone());
      }
      results.push(row_symbols);
    }
    results
  }

  #[cfg(test)]
  /// Get raw symbols of all cells, with printable placeholder for empty symbol ("").
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols_with_placeholder(&self) -> Vec<Vec<CompactString>> {
    let mut results: Vec<Vec<CompactString>> = vec![];
    for row in 0..self.size.height() {
      let mut row_symbols: Vec<CompactString> = vec![];
      for col in 0..self.size.width() {
        let idx = self.xy2idx(col as usize, row as usize);
        let s = self.cells[idx].symbol();
        row_symbols.push(if s.is_empty() {
          use compact_str::ToCompactString;
          " ".to_compact_string()
        } else {
          s.clone()
        });
      }
      results.push(row_symbols);
    }
    results
  }
}
