//! Canvas frame.

#![allow(dead_code)]

use compact_str::CompactString;
use geo::point;
use std::ops::Range;
use tracing::debug;

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

  // Utils {

  /// Convert start position and length of following N elements into Vec range.
  ///
  /// # Panics
  ///
  /// If the range is outside of frame shape.
  pub fn pos2range(&self, pos: U16Pos, n: usize) -> Range<usize> {
    assert_eq!(
      self.size.height() as usize * self.size.width() as usize,
      self.cells.len()
    );
    let start_idx = self.pos2idx(pos);
    let end_idx = start_idx + n;
    assert!(end_idx <= self.cells.len());
    start_idx..end_idx
  }

  /// Convert start index and length of following N elements into Vec range.
  ///
  /// # Panics
  ///
  /// If the range is outside of frame shape.
  pub fn idx2range(&self, index: usize, n: usize) -> Range<usize> {
    assert_eq!(
      self.size.height() as usize * self.size.width() as usize,
      self.cells.len()
    );
    let end_idx = index + n;
    assert!(end_idx <= self.cells.len());
    index..end_idx
  }

  /// Convert (position) X and Y into Vec index.
  ///
  /// # Panics
  ///
  /// If x and y is outside of frame shape.
  pub fn xy2idx(&self, x: usize, y: usize) -> usize {
    assert_eq!(
      self.size.height() as usize * self.size.width() as usize,
      self.cells.len()
    );
    let index = y * self.size.width() as usize + x;
    assert!(index <= self.cells.len());
    index
  }

  /// Convert position into Vec index.
  ///
  /// # Panics
  ///
  /// If position is outside of frame shape.
  pub fn pos2idx(&self, pos: U16Pos) -> usize {
    self.xy2idx(pos.x() as usize, pos.y() as usize)
  }

  // Utils }

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
    let index = self.pos2idx(pos);
    let result = &self.cells[index];
    debug!("get cell at index:{:?}, cell:{:?}", index, result);
    result
  }

  /// Set a cell.
  ///
  /// Returns the old cell.
  pub fn set_cell(&mut self, pos: U16Pos, cell: Cell) -> Cell {
    let index = self.pos2idx(pos);
    let old_cell = self.cells[index].clone();
    debug!(
      "set cell at index:{:?}, new cell:{:?}, old cell:{:?}",
      index, cell, old_cell
    );
    self.cells[index] = cell;
    let range = self.idx2range(index, 1);
    self.dirty_cells.push(range);
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
  ///
  /// # Panics
  ///
  /// If the range is outside of frame shape.
  pub fn cells_at(&self, pos: U16Pos, n: usize) -> &[Cell] {
    let range = self.pos2range(pos, n);
    &self.cells[range]
  }

  /// Get raw symbols of all cells.
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols_of_cells(&self) -> Vec<Vec<CompactString>> {
    let mut raw_symbols: Vec<Vec<CompactString>> = Vec::with_capacity(self.size.height() as usize);
    for row in 0..self.size.height() {
      let mut row_symbols: Vec<CompactString> = Vec::with_capacity(self.size.width() as usize);
      for col in 0..self.size.width() {
        let idx = self.xy2idx(col as usize, row as usize);
        row_symbols.push(self.cells[idx].symbol().clone());
      }
      raw_symbols.push(row_symbols);
    }
    raw_symbols
  }

  /// Set (replace) cells at a range.
  ///
  /// Returns old cells.
  ///
  /// # Panics
  ///
  /// If `cells` length exceed the canvas.
  pub fn set_cells_at(&mut self, pos: U16Pos, cells: Vec<Cell>) -> Vec<Cell> {
    let range = self.pos2range(pos, cells.len());
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

  /// Get (first,last) boundary positions by row. The `first` is the left position of the row,
  /// the `last` is the right position of the row.
  ///
  /// The `row` parameter starts from 0. NOTE: Row is X-axis.
  ///
  /// # Returns
  ///
  /// 1. Returns a pair/tuple of two positions, i.e. first and last positions, if the frame has
  ///    this row.
  /// 2. Returns `None`, if the frame is zero-sized or it doesn't have this row.
  pub fn row_boundary(&self, row: u16) -> Option<(U16Pos, U16Pos)> {
    if self.size.width() > 0 && self.size.height() > 0 && self.size.height() > row {
      Some((
        point!(x: 0_u16, y: row),
        point!(x: self.size.width()-1, y: row),
      ))
    } else {
      None
    }
  }

  /// Get (first,last) boundary positions by column. The `first` is the top position of the column,
  /// the `last` is the bottom position of the column.
  ///
  /// The `col` parameter starts from 0. NOTE: Column is Y-axis.
  ///
  /// # Returns
  ///
  /// 1. Returns a pair/tuple of two positions, i.e. first and last positions, if the frame has
  ///    this column.
  /// 2. Returns `None`, if the frame is zero-sized or it doesn't have this column.
  pub fn column_boundary(&self, col: u16) -> Option<(U16Pos, U16Pos)> {
    if self.size.height() > 0 && self.size.width() > 0 && self.size.width() > col {
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
  use compact_str::ToCompactString;
  use crossterm::style::{Attributes, Color};
  use std::sync::Once;
  use tracing::info;

  use super::*;
  use crate::test::log::init as test_log_init;

  static INIT: Once = Once::new();

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
  fn pos2range1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, Cursor::default());
    assert_eq!(frame.pos2range(point!(x: 0, y:0), 7), 0..7);
    assert_eq!(frame.pos2range(point!(x: 7, y:2), 23), 27..50);
    assert_eq!(frame.pos2range(point!(x: 8, y:9), 1), 98..99);
    assert_eq!(frame.pos2range(point!(x: 9, y:9), 1), 99..100);
  }

  #[test]
  fn idx2range1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, Cursor::default());
    assert_eq!(frame.idx2range(0, 7), 0..7);
    assert_eq!(frame.idx2range(27, 23), 27..50);
    assert_eq!(frame.idx2range(98, 1), 98..99);
    assert_eq!(frame.idx2range(99, 1), 99..100);
  }

  #[test]
  fn xy2idx1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, Cursor::default());
    assert_eq!(frame.xy2idx(0, 7), 70);
    assert_eq!(frame.xy2idx(7, 3), 37);
    assert_eq!(frame.xy2idx(1, 0), 1);
    assert_eq!(frame.xy2idx(0, 9), 90);
    assert_eq!(frame.xy2idx(9, 9), 99);
  }

  #[test]
  fn pos2idx1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, Cursor::default());
    assert_eq!(frame.pos2idx(point!(x:0, y:7)), 70);
    assert_eq!(frame.pos2idx(point!(x:7, y:3)), 37);
    assert_eq!(frame.pos2idx(point!(x:1, y:0)), 1);
    assert_eq!(frame.pos2idx(point!(x:0, y:9)), 90);
    assert_eq!(frame.pos2idx(point!(x:9, y:9)), 99);
  }

  #[test]
  fn set_cell1() {
    INIT.call_once(test_log_init);
    let frame_size = U16Size::new(10, 10);
    let mut frame = Frame::new(frame_size, Cursor::default());

    let inputs: Vec<(U16Pos, char)> = vec![
      (point!(x: 0, y: 0), 'A'),
      (point!(x: 7, y: 8), 'B'),
      (point!(x: 1, y: 3), 'C'),
      (point!(x: 9, y: 2), 'D'),
      (point!(x: 9, y: 9), 'E'),
      (point!(x: 2, y: 9), 'F'),
      (point!(x: 9, y: 7), 'G'),
    ];

    for (i, input) in inputs.iter().enumerate() {
      let mut c = Cell::default();
      c.set_symbol(input.1.to_compact_string());
      let actual = frame.set_cell(input.0, c);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), CompactString::new(""));
      assert_eq!(actual.fg(), Color::Reset);
      assert_eq!(actual.bg(), Color::Reset);
      assert_eq!(actual.attrs(), Attributes::default());
    }
    for (i, input) in inputs.iter().enumerate() {
      let actual = frame.cell(input.0);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), input.1.to_compact_string());
      assert_eq!(actual.fg(), Color::Reset);
      assert_eq!(actual.bg(), Color::Reset);
      assert_eq!(actual.attrs(), Attributes::default());
    }
  }

  #[test]
  fn set_empty_cell1() {
    INIT.call_once(test_log_init);
    let frame_size = U16Size::new(10, 10);
    let mut frame = Frame::new(frame_size, Cursor::default());

    let inputs: Vec<(U16Pos, char)> = vec![
      (point!(x: 0, y: 0), 'A'),
      (point!(x: 7, y: 8), 'B'),
      (point!(x: 1, y: 3), 'C'),
      (point!(x: 9, y: 2), 'D'),
      (point!(x: 9, y: 9), 'E'),
      (point!(x: 2, y: 9), 'F'),
      (point!(x: 9, y: 7), 'G'),
    ];

    for (i, input) in inputs.iter().enumerate() {
      let mut c = Cell::default();
      c.set_symbol(input.1.to_compact_string());
      let actual = frame.set_cell(input.0, c);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), CompactString::new(""));
      assert_eq!(actual.fg(), Color::Reset);
      assert_eq!(actual.bg(), Color::Reset);
      assert_eq!(actual.attrs(), Attributes::default());
    }
    for (i, input) in inputs.iter().enumerate() {
      let actual = frame.cell(input.0);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), input.1.to_compact_string());
      assert_eq!(actual.fg(), Color::Reset);
      assert_eq!(actual.bg(), Color::Reset);
      assert_eq!(actual.attrs(), Attributes::default());
    }
    for (i, input) in inputs.iter().enumerate() {
      let mut c = Cell::default();
      c.set_symbol(input.1.to_compact_string());
      let actual = frame.set_empty_cell(input.0);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), input.1.to_compact_string());
    }
    for (i, input) in inputs.iter().enumerate() {
      let actual = frame.cell(input.0);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), CompactString::new(""));
    }
  }

  #[test]
  fn cells_at1() {
    INIT.call_once(test_log_init);
    let frame_size = U16Size::new(10, 10);
    let mut frame = Frame::new(frame_size, Cursor::default());

    let inputs: Vec<(U16Pos, char)> = vec![
      (point!(x: 0, y: 0), 'A'),
      (point!(x: 7, y: 1), 'B'),
      (point!(x: 1, y: 2), 'C'),
      (point!(x: 6, y: 3), 'D'),
      (point!(x: 5, y: 4), 'E'),
      (point!(x: 4, y: 5), 'F'),
      (point!(x: 2, y: 6), 'G'),
      (point!(x: 0, y: 7), 'H'),
      (point!(x: 9, y: 8), 'I'),
      (point!(x: 3, y: 9), 'J'),
    ];
    let expects = [
      "A         ",
      "       B  ",
      " C        ",
      "      D   ",
      "     E    ",
      "    F     ",
      "  G       ",
      "H         ",
      "         I",
      "   J      ",
    ];

    for (i, input) in inputs.iter().enumerate() {
      let mut c = Cell::default();
      c.set_symbol(input.1.to_compact_string());
      let actual = frame.set_cell(input.0, c);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), CompactString::new(""));
    }
    info!("1-raw_symbols_of_cells:{:?}", frame.raw_symbols_of_cells(),);
    for i in 0..10 {
      let pos: U16Pos = point!(x:0, y:i);
      let cells = frame.cells_at(pos, 10);
      let actual = cells
        .iter()
        .map(|c| {
          if c.symbol().is_empty() {
            " ".to_string()
          } else {
            c.symbol().to_string()
          }
        })
        .collect::<Vec<_>>()
        .join("");
      let expect = expects[i as usize];
      info!("{i:?} pos:{pos:?}, cells:{cells:?}, actual:{actual:?}, expect:{expect:?}");
      assert_eq!(actual, expect);
    }

    let actual = frame
      .raw_symbols_of_cells()
      .iter()
      .map(|sv| {
        sv.iter()
          .map(|c| {
            if c.is_empty() {
              " ".to_string()
            } else {
              c.to_string()
            }
          })
          .collect::<Vec<_>>()
          .join("")
      })
      .collect::<Vec<_>>();
    info!(
      "2-raw_symbols_of_cells:{:?}, actual:{:?}",
      frame.raw_symbols_of_cells(),
      actual
    );
    assert_eq!(expects.len(), actual.len());
    for (i, expect) in expects.iter().enumerate() {
      let a = actual[i].clone();
      assert_eq!(a, expect.to_string());
    }
  }

  #[test]
  fn row_boundary1() {
    INIT.call_once(test_log_init);
    let sizes: Vec<U16Size> = [(10, 20), (20, 7), (13, 18), (15, 15), (0, 0)]
      .into_iter()
      .map(|(width, height)| U16Size::new(width, height))
      .collect();
    for frame_size in sizes.into_iter() {
      let frame = Frame::new(frame_size, Cursor::default());
      for row in 0..(2 * frame_size.height() + 10) {
        let actual = frame.row_boundary(row);
        info!(
          "frame size:{:?}, row:{:?}, actual:{:?}",
          frame_size, row, actual
        );
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

  #[test]
  fn column_boundary1() {
    INIT.call_once(test_log_init);
    let sizes: Vec<U16Size> = [(10, 20), (20, 7), (13, 18), (15, 15), (0, 0)]
      .into_iter()
      .map(|(width, height)| U16Size::new(width, height))
      .collect();
    for frame_size in sizes.into_iter() {
      let frame = Frame::new(frame_size, Cursor::default());
      for column in 0..(2 * frame_size.width() + 10) {
        let actual = frame.column_boundary(column);
        info!(
          "frame size:{:?}, column:{:?}, actual:{:?}",
          frame_size, column, actual
        );
        if column >= frame_size.width() {
          assert!(actual.is_none());
        } else {
          assert!(actual.is_some());
          let (start_at, end_at) = actual.unwrap();
          assert_eq!(start_at.x(), column);
          assert_eq!(start_at.y(), 0);
          assert_eq!(end_at.x(), column);
          assert_eq!(end_at.y(), frame_size.height() - 1);
        }
      }
    }
  }
}
