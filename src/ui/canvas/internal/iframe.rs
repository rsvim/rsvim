//! Internal frame implementation.

use compact_str::CompactString;
use geo::point;
use std::ops::Range;
// use tracing::debug;

use crate::cart::{U16Pos, U16Size};
use crate::ui::canvas::frame::cell::Cell;

#[derive(Debug, Clone)]
/// Internal iframe implementation for the Frame.
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
    assert!(index <= self.cells.len());
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

  /// Try get a cell.
  pub fn try_get_cell(&self, pos: U16Pos) -> Option<&Cell> {
    let index = self.pos2idx(pos);
    if self.contains_index(index) {
      let result = &self.cells[index];
      // debug!(
      //   "try get cell at pos:{:?}, index:{:?}, cell:{:?}",
      //   pos, index, result
      // );
      Some(result)
    } else {
      // debug!("try get cell invalid at pos:{:?}, index:{:?}", pos, index);
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

  /// Try set a cell.
  pub fn try_set_cell(&mut self, pos: U16Pos, cell: Cell) -> Option<Cell> {
    let index = self.pos2idx(pos);
    if self.contains_index(index) {
      let old_cell = self.cells[index].clone();
      // debug!(
      //   "try set cell at index:{:?}, new cell:{:?}, old cell:{:?}",
      //   index, cell, old_cell
      // );
      self.cells[index] = cell;
      self.dirty_rows[pos.y() as usize] = true;
      Some(old_cell)
    } else {
      // debug!("try set cell invalid index:{:?}, cell:{:?}", index, cell);
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

  /// Try set an empty cell.
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

  /// Try get a range of continuously cells.
  pub fn try_get_cells_at(&self, pos: U16Pos, n: usize) -> Option<&[Cell]> {
    let range = self.pos2range(pos, n);
    if self.contains_range(&range) {
      Some(&self.cells[range])
    } else {
      // debug!("try get cells at invalid range:{:?}", range);
      None
    }
  }

  /// Get raw symbols of all cells.
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols(&self) -> Vec<Vec<CompactString>> {
    let mut results: Vec<Vec<CompactString>> = Vec::with_capacity(self.size.height() as usize);
    for row in 0..self.size.height() {
      let mut row_symbols: Vec<CompactString> = Vec::with_capacity(self.size.width() as usize);
      for col in 0..self.size.width() {
        let idx = self.xy2idx(col as usize, row as usize);
        row_symbols.push(self.cells[idx].symbol().clone());
      }
      results.push(row_symbols);
    }
    results
  }

  /// Get raw symbols of all cells, with printable placeholder for empty symbol ("").
  ///
  /// NOTE: This method is mostly for debugging and testing.
  pub fn raw_symbols_with_placeholder(&self, printable: CompactString) -> Vec<Vec<CompactString>> {
    let mut results: Vec<Vec<CompactString>> = Vec::with_capacity(self.size.height() as usize);
    for row in 0..self.size.height() {
      let mut row_symbols: Vec<CompactString> = Vec::with_capacity(self.size.width() as usize);
      for col in 0..self.size.width() {
        let idx = self.xy2idx(col as usize, row as usize);
        let s = self.cells[idx].symbol();
        row_symbols.push(if s.is_empty() {
          printable.clone()
        } else {
          s.clone()
        });
      }
      results.push(row_symbols);
    }
    results
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

  /// Try set (replace) cells at a range.
  pub fn try_set_cells_at(&mut self, pos: U16Pos, cells: Vec<Cell>) -> Option<Vec<Cell>> {
    let range = self.pos2range(pos, cells.len());
    // debug!(
    //   "try set cells at range:{:?}, cells len:{:?}",
    //   range,
    //   self.cells.len()
    // );
    if self.contains_range(&range) {
      let end_at = self.idx2pos(range.end);
      // debug!("try set dirty rows for pos:{:?}, end_at:{:?}", pos, end_at);
      for row in pos.y()..(end_at.y() + 1) {
        // debug!("try set dirty rows at row:{:?}", row);
        if (row as usize) < self.dirty_rows.len() {
          self.dirty_rows[row as usize] = true;
        }
      }
      // debug!(
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

  /// Try set (replace) empty cells at a range.
  pub fn try_set_empty_cells_at(&mut self, pos: U16Pos, n: usize) -> Option<Vec<Cell>> {
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
mod tests {
  use compact_str::ToCompactString;
  use crossterm::style::{Attributes, Color};
  use tracing::info;

  use super::*;
  // use crate::test::log::init as test_log_init;

  #[test]
  fn new1() {
    let sz = U16Size::new(2, 1);
    let f = Iframe::new(sz);
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
    let frame = Iframe::new(frame_size);
    assert_eq!(frame.pos2range(point!(x: 0, y:0), 7), 0..7);
    assert_eq!(frame.pos2range(point!(x: 7, y:2), 23), 27..50);
    assert_eq!(frame.pos2range(point!(x: 8, y:9), 1), 98..99);
    assert_eq!(frame.pos2range(point!(x: 9, y:9), 1), 99..100);
  }

  #[test]
  fn idx2range1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Iframe::new(frame_size);
    assert_eq!(frame.idx2range(0, 7), 0..7);
    assert_eq!(frame.idx2range(27, 23), 27..50);
    assert_eq!(frame.idx2range(98, 1), 98..99);
    assert_eq!(frame.idx2range(99, 1), 99..100);
  }

  #[test]
  fn xy2idx1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Iframe::new(frame_size);
    assert_eq!(frame.xy2idx(0, 7), 70);
    assert_eq!(frame.xy2idx(7, 3), 37);
    assert_eq!(frame.xy2idx(1, 0), 1);
    assert_eq!(frame.xy2idx(0, 9), 90);
    assert_eq!(frame.xy2idx(9, 9), 99);
  }

  #[test]
  fn pos2idx1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Iframe::new(frame_size);
    assert_eq!(frame.pos2idx(point!(x:0, y:7)), 70);
    assert_eq!(frame.pos2idx(point!(x:7, y:3)), 37);
    assert_eq!(frame.pos2idx(point!(x:1, y:0)), 1);
    assert_eq!(frame.pos2idx(point!(x:0, y:9)), 90);
    assert_eq!(frame.pos2idx(point!(x:9, y:9)), 99);
  }

  #[test]
  fn idx2xy1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Iframe::new(frame_size);
    assert_eq!(frame.idx2xy(70), (0, 7));
    assert_eq!(frame.idx2xy(37), (7, 3));
    assert_eq!(frame.idx2xy(1), (1, 0));
    assert_eq!(frame.idx2xy(90), (0, 9));
    assert_eq!(frame.idx2xy(99), (9, 9));
  }

  #[test]
  fn idx2pos1() {
    let frame_size = U16Size::new(10, 10);
    let frame = Iframe::new(frame_size);
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
    let mut frame = Iframe::new(frame_size);

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
    let mut frame = Iframe::new(frame_size);

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
    let mut frame = Iframe::new(frame_size);

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
    let mut frame = Iframe::new(frame_size);

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
