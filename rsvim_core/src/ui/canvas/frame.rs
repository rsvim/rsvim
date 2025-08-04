//! Frame inside the canvas.

use crate::prelude::*;
use crate::ui::canvas::frame::cell::Cell;
use crate::ui::canvas::frame::cursor::Cursor;
use crate::ui::canvas::internal::iframe::Iframe;

use geo::point;
use std::ops::Range;

pub mod cell;
pub mod cursor;

#[cfg(test)]
mod cursor_tests;

#[derive(Debug, Clone)]
/// Logical frame for the canvas.
///
/// When UI widget tree drawing on the canvas, it actually draws on the current frame. Then the
/// canvas will diff the changes made by UI tree, and only print the changes to hardware device.
pub struct Frame {
  /// Iframe
  iframe: Iframe,
  /// Cursor
  cursor: Cursor,
}

impl Frame {
  /// Make new frame.
  pub fn new(size: U16Size, cursor: Cursor) -> Self {
    Frame {
      iframe: Iframe::new(size),
      cursor,
    }
  }

  // Utils {

  /// Convert start position and length of following N elements into Vec range.
  ///
  /// Returns the left-inclusive right-exclusive index range.
  pub fn pos2range(&self, pos: U16Pos, n: usize) -> Range<usize> {
    self.iframe.pos2range(pos, n)
  }

  /// Convert start index and length of following N elements into Vec range.
  pub fn idx2range(&self, index: usize, n: usize) -> Range<usize> {
    self.iframe.idx2range(index, n)
  }

  /// Convert (position) X and Y into Vec index.
  pub fn xy2idx(&self, x: usize, y: usize) -> usize {
    self.iframe.xy2idx(x, y)
  }

  /// Convert position into Vec index.
  pub fn pos2idx(&self, pos: U16Pos) -> usize {
    self.iframe.pos2idx(pos)
  }

  /// Convert index into (position) X and Y.
  ///
  /// Returns `(x, y)`.
  ///
  /// # Panics
  ///
  /// If index is outside of frame shape.
  pub fn idx2xy(&self, index: usize) -> (usize, usize) {
    self.iframe.idx2xy(index)
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
    self.iframe.size()
  }

  /// Whether the frame is zero sized.
  pub fn zero_sized(&self) -> bool {
    self.iframe.zero_sized()
  }

  /// Set current frame size.
  pub fn set_size(&mut self, size: U16Size) -> U16Size {
    self.iframe.set_size(size)
  }

  /// Whether index is inside frame cells.
  pub fn contains_index(&self, index: usize) -> bool {
    self.iframe.contains_index(index)
  }

  /// Whether range is inside frame cells. The range is left-inclusive, right-exclusive.
  pub fn contains_range(&self, range: &Range<usize>) -> bool {
    self.iframe.contains_range(range)
  }

  /// Get a cell.
  ///
  /// # Panics
  ///
  /// If the position is outside of frame shape.
  pub fn get_cell(&self, pos: U16Pos) -> &Cell {
    self.iframe.get_cell(pos)
  }

  /// Try get a cell, non-panic version of [`get_cell`](Frame::get_cell).
  pub fn try_get_cell(&self, pos: U16Pos) -> Option<&Cell> {
    self.iframe.try_get_cell(pos)
  }

  /// Set a cell.
  ///
  /// Returns the old cell.
  ///
  /// # Panics
  ///
  /// If the position is outside of frame shape.
  pub fn set_cell(&mut self, pos: U16Pos, cell: Cell) -> Cell {
    self.iframe.set_cell(pos, cell)
  }

  /// Try set a cell, non-panic version of [`set_cell`](Frame::set_cell).
  pub fn try_set_cell(&mut self, pos: U16Pos, cell: Cell) -> Option<Cell> {
    self.iframe.try_set_cell(pos, cell)
  }

  /// Set an empty cell.
  ///
  /// Returns the old cell.
  ///
  /// # Panics
  ///
  /// If the position is outside of frame shape.
  pub fn set_empty_cell(&mut self, pos: U16Pos) -> Cell {
    self.iframe.set_empty_cell(pos)
  }

  /// Try set an empty cell, non-panic version of [`set_empty_cell`](Frame::set_empty_cell).
  pub fn try_set_empty_cell(&mut self, pos: U16Pos) -> Option<Cell> {
    self.iframe.try_set_empty_cell(pos)
  }

  /// Get all cells.
  pub fn get_cells(&self) -> &Vec<Cell> {
    self.iframe.get_cells()
  }

  /// Get a range of continuously cells.
  ///
  /// # Panics
  ///
  /// If the range is outside of frame shape.
  pub fn get_cells_at(&self, pos: U16Pos, n: usize) -> &[Cell] {
    self.iframe.get_cells_at(pos, n)
  }

  /// Try get a range of continuously cells, non-panic version of
  /// [`get_cells_at`](Frame::get_cells_at).
  pub fn try_get_cells_at(&self, pos: U16Pos, n: usize) -> Option<&[Cell]> {
    self.iframe.try_get_cells_at(pos, n)
  }

  /// Set (replace) cells at a range.
  ///
  /// Returns old cells.
  ///
  /// # Panics
  ///
  /// If any positions of `cells` is outside of frame shape.
  pub fn set_cells_at(&mut self, pos: U16Pos, cells: Vec<Cell>) -> Vec<Cell> {
    self.iframe.set_cells_at(pos, cells)
  }

  /// Try set (replace) cells at a range, non-panic version of
  /// [`set_cells_at`](Frame::set_cells_at).
  pub fn try_set_cells_at(
    &mut self,
    pos: U16Pos,
    cells: Vec<Cell>,
  ) -> Option<Vec<Cell>> {
    self.iframe.try_set_cells_at(pos, cells)
  }

  /// Set (replace) empty cells at a range.
  ///
  /// # Panics
  ///
  /// If any positions of `cells` is outside of frame shape.
  pub fn set_empty_cells_at(&mut self, pos: U16Pos, n: usize) -> Vec<Cell> {
    self.iframe.set_empty_cells_at(pos, n)
  }

  /// Try set (replace) empty cells at a range, non-panic version of
  /// [`set_empty_cells_at`](Frame::set_empty_cells_at).
  pub fn try_set_empty_cells_at(
    &mut self,
    pos: U16Pos,
    n: usize,
  ) -> Option<Vec<Cell>> {
    self.iframe.try_set_empty_cells_at(pos, n)
  }

  /// Get dirty rows.
  pub fn dirty_rows(&self) -> &Vec<bool> {
    self.iframe.dirty_rows()
  }

  /// Reset/clean all dirty components.
  ///
  /// NOTE: This method should be called after current frame flushed to terminal device.
  pub fn reset_dirty_rows(&mut self) {
    self.iframe.reset_dirty_rows()
  }

  /// Get cursor.
  pub fn cursor(&self) -> &Cursor {
    &self.cursor
  }

  /// Set cursor.
  pub fn set_cursor(&mut self, cursor: Cursor) {
    self.cursor = cursor;
  }
}

#[cfg(test)]
use compact_str::CompactString;

impl Frame {
  #[cfg(test)]
  /// Get raw symbols of all cells.
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols(&self) -> Vec<Vec<CompactString>> {
    self.iframe.raw_symbols()
  }

  #[cfg(test)]
  /// Get raw symbols of all cells, with printable placeholder for empty symbol ("").
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols_with_placeholder(&self) -> Vec<Vec<CompactString>> {
    self.iframe.raw_symbols_with_placeholder()
  }
}
