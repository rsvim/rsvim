//! Canvas frame.

#![allow(dead_code)]

use geo::point;
use std::ops::Range;

use crate::cart::{U16Pos, U16Size};
use crate::ui::canvas::frame::cell::Cell;
use crate::ui::canvas::frame::cursor::Cursor;

pub mod cell;
pub mod cursor;

#[derive(Debug, Clone)]
/// Logical frame for the canvas.
///
/// When UI widget tree drawing on the canvas, it actually draws on the current frame. Then the
/// canvas will diff the changes made by UI tree, and only print the changes to hardware device.
pub struct Frame {
  size: U16Size,

  /// Cells
  cells: Vec<Cell>,

  /// Indicate which part of the frame is dirty.
  ///
  /// NOTE: This is only for fast locating the changed parts, but can be false positive, i.e. if a
  /// location is marked in this collection, it can still be unchanged. But if a location is not
  /// marked in this collection, it must be unchanged.
  dirty_cells: Vec<Range<usize>>,

  /// Cursor
  cursor: Cursor,

  /// Indicate whether the cursor is changed.
  dirty_cursor: bool,
}

// Make range from start position and length of following N elements.
fn range_from_pos(pos: U16Pos, n: usize) -> Range<usize> {
  let start_at = pos.x() as usize * pos.y() as usize;
  let end_at = start_at + n;
  start_at..end_at
}

// Make range from start index and length of following N elements.
fn range_from_idx(index: usize, n: usize) -> Range<usize> {
  let end_at = index + n;
  index..end_at
}

impl Frame {
  /// Make new frame.
  pub fn new(size: U16Size, cursor: Cursor) -> Self {
    let n = size.height() as usize * size.width() as usize;
    Frame {
      size,
      cells: vec![Cell::default(); n],
      dirty_cells: vec![], // When first create, it's not dirty.
      cursor,
      dirty_cursor: false,
    }
  }

  /// Get current frame size.
  pub fn size(&self) -> U16Size {
    self.size
  }

  /// Set current frame size.
  pub fn set_size(&mut self, size: U16Size) -> U16Size {
    let old_size = self.size;
    self.size = size;
    self.cells.resize(
      size.height() as usize * size.width() as usize,
      Cell::default(),
    );
    old_size
  }

  /// Get a cell.
  pub fn cell(&self, pos: U16Pos) -> &Cell {
    &self.cells[pos.x() as usize * pos.y() as usize]
  }

  /// Set a cell.
  ///
  /// Returns the old cell.
  pub fn set_cell(&mut self, pos: U16Pos, cell: Cell) -> Cell {
    let index = pos.x() as usize * pos.y() as usize;
    let old_cell = self.cells[index].clone();
    self.cells[index] = cell;
    self.dirty_cells.push(range_from_idx(index, 1));
    old_cell
  }

  /// Set an empty cell.
  ///
  /// Returns the old cell.
  pub fn set_empty_cell(&mut self, pos: U16Pos) -> Cell {
    self.set_cell(pos, Cell::empty())
  }

  /// Get all cells.
  pub fn cells(&self) -> &Vec<Cell> {
    &self.cells
  }

  /// Get a range of continuously cells, start from a position and last for N elements.
  pub fn cells_at(&self, pos: U16Pos, n: usize) -> &[Cell] {
    &self.cells[range_from_pos(pos, n)]
  }

  /// Set (replace) cells at a range.
  ///
  /// Returns old cells.
  ///
  /// # Panics
  ///
  /// If `cells` length exceed the canvas.
  pub fn set_cells_at(&mut self, pos: U16Pos, cells: Vec<Cell>) -> Vec<Cell> {
    let range = range_from_pos(pos, cells.len());
    assert!(range.end <= self.cells.len());
    self.dirty_cells.push(range.clone());
    self.cells.splice(range, cells).collect()
  }

  /// Set (replace) empty cells at a range.
  pub fn set_empty_cells_at(&mut self, pos: U16Pos, n: usize) -> Vec<Cell> {
    self.set_cells_at(pos, vec![Cell::empty(); n])
  }

  /// Get dirty cells.
  pub fn dirty_cells(&self) -> &Vec<Range<usize>> {
    &self.dirty_cells
  }

  /// Get cursor.
  pub fn cursor(&self) -> &Cursor {
    &self.cursor
  }

  /// Set cursor.
  pub fn set_cursor(&mut self, cursor: Cursor) {
    if self.cursor != cursor {
      self.cursor = cursor;
      self.dirty_cursor = true;
    }
  }

  /// Whether cursor is dirty.
  pub fn dirty_cursor(&self) -> bool {
    self.dirty_cursor
  }

  /// Reset/clean all dirty components.
  ///
  /// NOTE: This method should be called after current frame flushed to terminal device.
  pub fn reset_dirty(&mut self) {
    self.dirty_cells = vec![];
    self.dirty_cursor = false;
  }

  // Rows/Columns Helper {

  /// Get (start,end) boundary positions by row. The `start` is the left position of the row,
  /// `end` is the right position of the row.
  ///
  /// The `row` parameter starts from 0.
  ///
  /// # Returns
  ///
  /// 1. Returns a pair/tuple of two positions, i.e. start and end positions, if the frame has this
  ///    row.
  /// 2. Returns `None`, if the frame is zero-sized or it doesn't have this row.
  pub fn row_boundary(&self, row: u16) -> Option<(U16Pos, U16Pos)> {
    if self.size.width() > 0 && self.size.height() > 0 && self.size.height() >= row {
      Some((
        point!(x: 0_u16, y: row),
        point!(x: self.size.width()-1, y: row),
      ))
    } else {
      None
    }
  }

  /// Get (start,end) boundary positions by column. The `start` is the top position of the column,
  /// the `end` is the bottom position of the column.
  ///
  /// The `col` parameter starts from 0.
  ///
  /// # Returns
  ///
  /// 1. Returns a pair/tuple of two positions, i.e. start and end positions, if the frame has this
  ///    row.
  /// 2. Returns `None`, if the frame is zero-sized or it doesn't have this row.
  pub fn column_boundary(&self, col: u16) -> Option<(U16Pos, U16Pos)> {
    if self.size.height() > 0 && self.size.width() > 0 && self.size.width() >= col {
      Some((
        point!(x: col, y: 0_u16),
        point!(x: col, y: self.size.height()-1),
      ))
    } else {
      None
    }
  }

  // Rows/Columns Helper }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new1() {
    let sz = U16Size::new(2, 1);
    let f = Frame::new(sz, Cursor::default());
    assert_eq!(f.size.width(), 2);
    assert_eq!(f.size.height(), 1);
    assert_eq!(
      f.cells.len(),
      f.size.height() as usize * f.size.width() as usize
    );
    for c in f.cells.iter() {
      assert_eq!(c.symbol(), Cell::default().symbol());
    }
  }

  #[test]
  fn set_cells1() {
    let sz = U16Size::new(10, 10);
    let _f = Frame::new(sz, Cursor::default());
  }

  #[test]
  fn row_boundary1() {
    let sizes: Vec<U16Size> = [(10, 20), (20, 7), (13, 18), (15, 15), (0, 0)]
      .into_iter()
      .map(|(width, height)| U16Size::new(width, height))
      .collect();
    for frame_size in sizes.into_iter() {
      let frame = Frame::new(frame_size, Cursor::default());
      for column in 0..(2 * frame_size.width()) {
        let actual = frame.column_boundary(column);
        if column >= frame_size.width() {
          assert!(actual.is_none());
        } else {
          assert!(actual.is_some());
          let (start_at, end_at) = actual.unwrap();
          assert_eq!(start_at.x(), 0);
          assert_eq!(start_at.y(), column);
          assert_eq!(end_at.x(), frame_size.height() - 1);
          assert_eq!(end_at.y(), column);
        }
      }
    }
  }

  #[test]
  fn column_boundary1() {
    let sizes: Vec<U16Size> = [(10, 20), (20, 7), (13, 18), (15, 15), (0, 0)]
      .into_iter()
      .map(|(width, height)| U16Size::new(width, height))
      .collect();
    for frame_size in sizes.into_iter() {
      let frame = Frame::new(frame_size, Cursor::default());
      for row in 0..(2 * frame_size.height()) {
        let actual = frame.row_boundary(row);
        if row >= frame_size.height() {
          assert!(actual.is_none());
        } else {
          assert!(actual.is_some());
          let (start_at, end_at) = actual.unwrap();
          assert_eq!(start_at.x(), 0);
          assert_eq!(start_at.y(), row);
          assert_eq!(end_at.x(), frame_size.width() - 1);
          assert_eq!(end_at.y(), row);
        }
      }
    }
  }
}
