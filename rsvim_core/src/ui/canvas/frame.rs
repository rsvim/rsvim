//! Frame inside the canvas.

use compact_str::CompactString;
use geo::point;
use std::ops::Range;
// use tracing::debug;

use crate::cart::{U16Pos, U16Size};
use crate::ui::canvas::frame::cell::Cell;
use crate::ui::canvas::frame::cursor::CCursor;
use crate::ui::canvas::internal::iframe::Iframe;

pub mod cell;
pub mod cursor;

#[derive(Debug, Clone)]
/// Logical frame for the canvas.
///
/// When UI widget tree drawing on the canvas, it actually draws on the current frame. Then the
/// canvas will diff the changes made by UI tree, and only print the changes to hardware device.
pub struct Frame {
  /// Iframe
  iframe: Iframe,
  /// Cursor
  cursor: CCursor,
}

impl Frame {
  /// Make new frame.
  pub fn new(size: U16Size, cursor: CCursor) -> Self {
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

  /// Get raw symbols of all cells.
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols(&self) -> Vec<Vec<CompactString>> {
    self.iframe.raw_symbols()
  }

  /// Get raw symbols of all cells, with printable placeholder for empty symbol ("").
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols_with_placeholder(&self, printable: CompactString) -> Vec<Vec<CompactString>> {
    self.iframe.raw_symbols_with_placeholder(printable)
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
  pub fn try_set_cells_at(&mut self, pos: U16Pos, cells: Vec<Cell>) -> Option<Vec<Cell>> {
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
  pub fn try_set_empty_cells_at(&mut self, pos: U16Pos, n: usize) -> Option<Vec<Cell>> {
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
  pub fn cursor(&self) -> &CCursor {
    &self.cursor
  }

  /// Set cursor.
  pub fn set_cursor(&mut self, cursor: CCursor) {
    self.cursor = cursor;
  }
}

#[cfg(test)]
mod tests {
  use compact_str::ToCompactString;
  use crossterm::style::{Attributes, Color};
  use tracing::info;

  use super::*;
  // use crate::test::log::init as test_log_init;

  #[test]
  fn pos2range1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, CCursor::default());
    assert_eq!(frame.pos2range(point!(x: 0, y:0), 7), 0..7);
    assert_eq!(frame.pos2range(point!(x: 7, y:2), 23), 27..50);
    assert_eq!(frame.pos2range(point!(x: 8, y:9), 1), 98..99);
    assert_eq!(frame.pos2range(point!(x: 9, y:9), 1), 99..100);
  }

  #[test]
  fn idx2range1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, CCursor::default());
    assert_eq!(frame.idx2range(0, 7), 0..7);
    assert_eq!(frame.idx2range(27, 23), 27..50);
    assert_eq!(frame.idx2range(98, 1), 98..99);
    assert_eq!(frame.idx2range(99, 1), 99..100);
  }

  #[test]
  fn xy2idx1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, CCursor::default());
    assert_eq!(frame.xy2idx(0, 7), 70);
    assert_eq!(frame.xy2idx(7, 3), 37);
    assert_eq!(frame.xy2idx(1, 0), 1);
    assert_eq!(frame.xy2idx(0, 9), 90);
    assert_eq!(frame.xy2idx(9, 9), 99);
  }

  #[test]
  fn pos2idx1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, CCursor::default());
    assert_eq!(frame.pos2idx(point!(x:0, y:7)), 70);
    assert_eq!(frame.pos2idx(point!(x:7, y:3)), 37);
    assert_eq!(frame.pos2idx(point!(x:1, y:0)), 1);
    assert_eq!(frame.pos2idx(point!(x:0, y:9)), 90);
    assert_eq!(frame.pos2idx(point!(x:9, y:9)), 99);
  }

  #[test]
  fn idx2xy1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, CCursor::default());
    assert_eq!(frame.idx2xy(70), (0, 7));
    assert_eq!(frame.idx2xy(37), (7, 3));
    assert_eq!(frame.idx2xy(1), (1, 0));
    assert_eq!(frame.idx2xy(90), (0, 9));
    assert_eq!(frame.idx2xy(99), (9, 9));
  }

  #[test]
  fn idx2pos1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Frame::new(frame_size, CCursor::default());
    assert_eq!(frame.idx2pos(70), point!(x:0, y:7));
    assert_eq!(frame.idx2pos(37), point!(x:7, y:3));
    assert_eq!(frame.idx2pos(1), point!(x:1, y:0));
    assert_eq!(frame.idx2pos(90), point!(x:0, y:9));
    assert_eq!(frame.idx2pos(99), point!(x:9, y:9));
  }

  #[test]
  fn set_cell1() {
    // test_log_init();
    let frame_size = U16Size::new(10, 10);
    let mut frame = Frame::new(frame_size, CCursor::default());

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
      let actual = frame.get_cell(input.0);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), input.1.to_compact_string());
      assert_eq!(actual.fg(), Color::Reset);
      assert_eq!(actual.bg(), Color::Reset);
      assert_eq!(actual.attrs(), Attributes::default());
    }
  }

  #[test]
  fn set_empty_cell1() {
    // test_log_init();
    let frame_size = U16Size::new(10, 10);
    let mut frame = Frame::new(frame_size, CCursor::default());

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
      let actual = frame.get_cell(input.0);
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
      let actual = frame.get_cell(input.0);
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert_eq!(actual.symbol(), CompactString::new(""));
    }
  }

  #[test]
  fn cells_at1() {
    // test_log_init();
    let frame_size = U16Size::new(10, 10);
    let mut frame = Frame::new(frame_size, CCursor::default());

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
    info!("1-raw_symbols:{:?}", frame.raw_symbols(),);
    let all_cells = frame.get_cells();
    for i in 0..10 {
      let pos: U16Pos = point!(x:0, y:i);
      let cells = frame.get_cells_at(pos, 10);
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

      let idx = frame.pos2idx(pos);
      let cells = &all_cells[idx..(idx + 10)];
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
      assert_eq!(actual, expect);
    }

    let actual = frame
      .raw_symbols()
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
      "2-raw_symbols:{:?}, actual:{:?}",
      frame.raw_symbols(),
      actual
    );
    assert_eq!(expects.len(), actual.len());
    for (i, expect) in expects.iter().enumerate() {
      let a = actual[i].clone();
      assert_eq!(a, expect.to_string());
    }
  }

  #[test]
  fn set_cells_at1() {
    // test_log_init();
    let frame_size = U16Size::new(10, 10);
    let mut frame = Frame::new(frame_size, CCursor::default());

    let inputs: Vec<(U16Pos, &str)> = vec![
      (point!(x: 0, y: 0), "ABCD"),
      (point!(x: 7, y: 1), "EFGHIJK"),
      (point!(x: 1, y: 2), "LMN"),
      (point!(x: 6, y: 3), "OP"),
      (point!(x: 5, y: 4), "Q"),
      (point!(x: 4, y: 5), ""),
      (point!(x: 2, y: 6), "RSTUV"),
      (point!(x: 0, y: 7), "'WXYZ"),
      (point!(x: 9, y: 8), "abcdefghijk"),
      (point!(x: 3, y: 9), "opqrstu"),
    ];

    let expects = [
      "ABCD      ",
      "       EFG",
      "HLMN      ",
      "      OP  ",
      "     Q    ",
      "          ",
      "  RSTUV   ",
      "'WXYZ     ",
      "         a",
      "bcdopqrstu",
    ];

    for (i, input) in inputs.iter().enumerate() {
      let actual = frame.set_cells_at(input.0, input.1.chars().map(Cell::with_char).collect());
      info!("{:?} input:{:?}, actual:{:?}", i, input, actual);
      assert!(actual.len() == input.1.len());
    }
    let actuals = frame.raw_symbols_with_placeholder(" ".to_compact_string());
    assert_eq!(actuals.len(), expects.len());
    for (i, expect) in expects.into_iter().enumerate() {
      let actual = actuals[i].join("");
      info!("{:?} actual:{:?}, expect:{:?}", i, actual, expect);
      assert!(actual.len() == expect.len());
      assert_eq!(actual, expect);
    }
  }
}
