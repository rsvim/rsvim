//! Vim window's text content widget.

use crate::buf::{Buffer, BufferWk};
use crate::cart::{IRect, U16Pos, U16Rect, U16Size};
use crate::envar;
use crate::ui::canvas::internal::iframe::Iframe;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
use crate::ui::tree::Tree;
use crate::ui::util::ptr::SafeViewportRef;
use crate::ui::widget::window::viewport::Viewport;
use crate::ui::widget::window::{Window, WindowLocalOptions};
use crate::ui::widget::Widgetable;
use crate::{inode_generate_impl, rlock};

use crossterm::style::{Attributes, Color};
use geo::point;
use icu::segmenter::WordSegmenter;
use regex::Regex;
use ropey::RopeSlice;
use std::collections::{BTreeSet, VecDeque};
use std::convert::From;
use std::time::Duration;
use tracing::{debug, error};

#[derive(Debug, Clone)]
/// The widget contains text contents for Vim window.
pub struct WindowContent {
  base: InodeBase,

  buffer: BufferWk,

  viewport: Option<SafeViewportRef>,
}

impl WindowContent {
  /// Make window content.
  pub fn new(shape: IRect, buffer: BufferWk) -> Self {
    let base = InodeBase::new(shape);
    WindowContent {
      base,
      buffer,
      viewport: None,
    }
  }

  pub fn set_viewport(&mut self, viewport: &mut Viewport) {
    self.viewport = Some(SafeViewportRef::new(viewport));
  }
}

inode_generate_impl!(WindowContent, base);

#[allow(dead_code)]
fn rpslice2line(s: &RopeSlice) -> String {
  let mut builder: String = String::new();
  for chunk in s.chunks() {
    builder.push_str(chunk);
  }
  builder
}

fn truncate_line(line: &RopeSlice, max_chars: usize) -> String {
  let mut builder = String::new();
  builder.reserve(max_chars);
  for chunk in line.chunks() {
    if builder.len() > max_chars {
      return builder;
    }
    builder.push_str(chunk);
  }
  builder
}

// // Draw {
// impl WindowContent {
//   /// Draw buffer from `start_line`
//   pub fn _draw_from_top(
//     &mut self,
//     canvas: &mut Canvas,
//     start_line: usize,
//     start_column: usize,
//     end_column: usize,
//   ) {
//     match (self.wrap(), self.line_break()) {
//       (false, _) => self._draw_from_top_for_nowrap(canvas, start_line, start_column, end_column),
//       (true, false) => {
//         self._draw_from_top_for_wrap_nolinebreak(canvas, start_line, start_column, end_column)
//       }
//       (true, true) => {
//         self._draw_from_top_for_wrap_linebreak(canvas, start_line, start_column, end_column)
//       }
//     }
//   }
//
//   /// Implement the [`_draw_from_top`](WindowContent::_draw_from_top) with below options:
//   /// - [`warp`](WindowLocalOptions::wrap) is `true`.
//   /// - [`line_break`](WindowLocalOptions::line_break) is `true`
//   pub fn _draw_from_top_for_wrap_linebreak(
//     &mut self,
//     canvas: &mut Canvas,
//     start_line: usize,
//     _start_column: usize,
//     _end_column: usize,
//   ) {
//     let actual_shape = self.actual_shape();
//     let upos: U16Pos = actual_shape.min().into();
//     let height = actual_shape.height();
//     let width = actual_shape.width();
//
//     debug!("_draw_from_top_for_wrap_linebreak");
//     debug!(
//       "actual_shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
//       actual_shape, upos, height, width,
//     );
//
//     // If window is zero-sized.
//     if height == 0 || width == 0 {
//       return;
//     }
//
//     // Get buffer arc pointer
//     let buffer = self.buffer.upgrade().unwrap();
//
//     // Lock buffer for read
//     let buffer = buffer.try_read_for(envar::MUTEX_TIMEOUT()).unwrap();
//
//     if let Some(line) = buffer.get_line(start_line) {
//       debug!(
//         "buffer.get_line ({:?}):'{:?}'",
//         start_line,
//         rpslice2line(&line),
//       );
//     } else {
//       debug!("buffer.get_line ({:?}):None", start_line);
//     }
//
//     match buffer.get_lines_at(start_line) {
//       Some(mut buflines) => {
//         // The `start_line` is inside the buffer.
//         // Render the lines from `start_line` till the end of the buffer or the window widget.
//
//         // The first `row` (0) in the window maps to the `start_line` in the buffer.
//         let mut row = 0;
//         let segmenter = WordSegmenter::new_auto();
//
//         while row < height {
//           match buflines.next() {
//             Some(line) => {
//               // Chop the line into maximum chars to avoid super long lines for display.
//               let truncated_line = truncate_line(&line, height as usize * width as usize);
//               let breakpoints: Vec<usize> = segmenter.segment_str(&truncated_line).collect();
//               debug!(
//                 "1-truncated_line: {:?}, breakpoints: {:?}",
//                 truncated_line, breakpoints
//               );
//
//               let mut col = 0_u16;
//               for bp in 1..breakpoints.len() {
//                 if row >= height {
//                   break;
//                 }
//                 let word_start = breakpoints[bp - 1];
//                 let word_end = breakpoints[bp];
//                 let word = &truncated_line[word_start..word_end];
//                 let word_len = word_end - word_start;
//                 if word_len + col as usize <= width as usize {
//                   // Enough space to place this word in current row
//                   let cells = word.chars().map(Cell::from).collect::<Vec<_>>();
//                   let cells_upos = point!(x: col + upos.x(), y: row + upos.y());
//                   debug!(
//                     "2-row:{:?}, col:{:?}, cells:{:?}, cells_upos:{:?}",
//                     row,
//                     col,
//                     cells
//                       .iter()
//                       .map(|ch| ch.symbol().to_string())
//                       .collect::<Vec<String>>()
//                       .join(""),
//                     cells_upos
//                   );
//                   if word != "\n" {
//                     canvas.frame_mut().set_cells_at(cells_upos, cells);
//                     col += word_len as u16;
//                   }
//                 } else {
//                   // Not enough space to place this word in current row.
//                   // There're two cases:
//                   // 1. The word can be placed in next empty row (since the column idx `col` will
//                   //    start from 0 in next row).
//                   // 2. The word is just too long to place in an entire row, so next row still
//                   //    cannot place it.
//                   // Anyway, we simply go to next row, and force render all of the word.
//                   row += 1;
//                   col = 0_u16;
//
//                   for ch in word.chars() {
//                     if col >= width {
//                       row += 1;
//                       col = 0_u16;
//                     }
//                     if row >= height {
//                       break;
//                     }
//                     let cell = Cell::from(ch);
//                     let cell_upos = point!(x: col + upos.x(), y: row + upos.y());
//                     debug!(
//                       "3-row:{:?}, col:{:?}, ch:{:?}, cell_upos:{:?}",
//                       row, col, ch, cell_upos
//                     );
//                     if word != "\n" {
//                       canvas.frame_mut().set_cell(cell_upos, cell);
//                       col += 1;
//                     }
//                   }
//                 }
//               }
//             }
//             None => {
//               // If there's no more lines in the buffer, simply set the whole line to empty for
//               // left parts of the window.
//               let cells_upos = point!(x: upos.x(), y: row + upos.y());
//               let cells_len = width as usize;
//               debug!(
//                 "4-row:{:?}, cells upos:{:?}, cells len:{:?}",
//                 row, cells_upos, cells_len,
//               );
//               canvas
//                 .frame_mut()
//                 .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
//                 .unwrap();
//               row += 1;
//             }
//           }
//           row += 1;
//         }
//       }
//       None => {
//         // The `start_line` is outside of the buffer.
//         // Render the whole window contents as empty cells.
//
//         // The first `row` (0) in the window maps to the `start_line` in the buffer.
//         let mut row = 0;
//
//         while row < height {
//           // There's no lines in the buffer, simply set the whole line to empty.
//           let cells_upos = point!(x: upos.x(), y: row + upos.y());
//           let cells_len = width as usize;
//           debug!(
//             "5-row:{:?}, cells upos:{:?}, cells len:{:?}",
//             row, cells_upos, cells_len,
//           );
//           canvas
//             .frame_mut()
//             .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
//             .unwrap();
//           row += 1;
//         }
//       }
//     }
//   }
//
//   /// Implement the [`_draw_from_top`](WindowContent::_draw_from_top) with below options:
//   /// - [`warp`](WindowLocalOptions::wrap) is `true`.
//   /// - [`line_break`](WindowLocalOptions::line_break) is `false`.
//   pub fn _draw_from_top_for_wrap_nolinebreak(
//     &mut self,
//     canvas: &mut Canvas,
//     start_line: usize,
//     _start_column: usize,
//     _end_column: usize,
//   ) {
//     let actual_shape = self.actual_shape();
//     let upos: U16Pos = actual_shape.min().into();
//     let height = actual_shape.height();
//     let width = actual_shape.width();
//
//     debug!("_draw_from_top_for_wrap_nolinebreak");
//     // debug!(
//     //   "actual_shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
//     //   actual_shape, upos, height, width,
//     // );
//
//     // If window is zero-sized.
//     if height == 0 || width == 0 {
//       return;
//     }
//
//     // Get buffer arc pointer
//     let buffer = self.buffer.upgrade().unwrap();
//
//     // Lock buffer for read
//     let buffer = buffer.try_read_for(envar::MUTEX_TIMEOUT()).unwrap();
//
//     // if let Some(line) = buffer.rope().get_line(start_line) {
//     //   debug!(
//     //     "buffer.get_line ({:?}):'{:?}'",
//     //     start_line,
//     //     rpslice2line(&line),
//     //   );
//     // } else {
//     //   debug!("buffer.get_line ({:?}):None", start_line);
//     // }
//
//     match buffer.get_lines_at(start_line) {
//       Some(mut buflines) => {
//         // The `start_line` is inside the buffer.
//         // Render the lines from `start_line` till the end of the buffer or the window widget.
//
//         // The first `row` (0) in the window maps to the `start_line` in the buffer.
//         let mut row = 0;
//
//         while row < height {
//           match buflines.next() {
//             Some(line) => {
//               // For the row in current window widget, if has the line in buffer.
//               let mut col = 0_u16;
//
//               for chunk in line.chunks() {
//                 if col >= width {
//                   row += 1;
//                   col = 0_u16;
//                   if row >= height {
//                     break;
//                   }
//                 }
//                 for ch in chunk.chars() {
//                   if col >= width {
//                     row += 1;
//                     col = 0_u16;
//                     if row >= height {
//                       break;
//                     }
//                   }
//                   if ch != '\n' {
//                     let cell = Cell::from(ch);
//                     let cell_upos = point!(x: col + upos.x(), y: row + upos.y());
//                     // debug!(
//                     //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
//                     //   row, col, ch, cell_upos
//                     // );
//                     canvas.frame_mut().set_cell(cell_upos, cell);
//                   }
//                   col += 1;
//                 }
//               }
//
//               // The line doesn't fill the whole row in current widget, fill left parts with empty
//               // cells.
//               if row < height && col < width - 1 {
//                 let cells_upos = point!(x: col + upos.x(), y: row + upos.y());
//                 let cells_len = (width - col) as usize;
//                 // debug!(
//                 //   "2-row:{:?}, col:{:?}, cells upos:{:?}, cells len:{:?}",
//                 //   row, col, cells_upos, cells_len,
//                 // );
//                 canvas
//                   .frame_mut()
//                   .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
//                   .unwrap();
//               }
//             }
//             None => {
//               // If there's no more lines in the buffer, simply set the whole line to empty for
//               // left parts of the window.
//               let cells_upos = point!(x: upos.x(), y: row + upos.y());
//               let cells_len = width as usize;
//               // debug!(
//               //   "3-row:{:?}, cells upos:{:?}, cells len:{:?}",
//               //   row, cells_upos, cells_len,
//               // );
//               canvas
//                 .frame_mut()
//                 .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
//                 .unwrap();
//             }
//           }
//           // Iterate to next row.
//           row += 1;
//         }
//       }
//       None => {
//         // The `start_line` is outside of the buffer.
//         // Render the whole window contents as empty cells.
//
//         // The first `row` (0) in the window maps to the `start_line` in the buffer.
//         let mut row = 0;
//
//         while row < height {
//           // There's no lines in the buffer, simply set the whole line to empty.
//           let cells_upos = point!(x: upos.x(), y: row + upos.y());
//           let cells_len = width as usize;
//           // debug!(
//           //   "4-row:{:?}, cells upos:{:?}, cells len:{:?}",
//           //   row, cells_upos, cells_len,
//           // );
//           canvas
//             .frame_mut()
//             .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
//             .unwrap();
//           row += 1;
//         }
//       }
//     }
//   }
//
//   /// Implement the [`_draw_from_top`](WindowContent::_draw_from_top) with below options:
//   /// - [`warp`](WindowLocalOptions::wrap) is `false`.
//   /// - [`line_break`](WindowLocalOptions::line_break) is not be used.
//   pub fn _draw_from_top_for_nowrap(
//     &mut self,
//     canvas: &mut Canvas,
//     start_line: usize,
//     _start_column: usize,
//     _end_column: usize,
//   ) {
//     let actual_shape = self.actual_shape();
//     let upos: U16Pos = actual_shape.min().into();
//     let height = actual_shape.height();
//     let width = actual_shape.width();
//
//     debug!("_draw_from_top_for_nowrap");
//     // debug!(
//     //   "actual shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
//     //   actual_shape, upos, height, width,
//     // );
//
//     // If window is zero-sized.
//     if height == 0 || width == 0 {
//       return;
//     }
//
//     // Get buffer arc pointer
//     let buffer = self.buffer.upgrade().unwrap();
//
//     // Lock buffer for read
//     let buffer = buffer.try_read_for(envar::MUTEX_TIMEOUT()).unwrap();
//
//     // if let Some(line) = buffer.rope().get_line(start_line) {
//     //   debug!(
//     //     "buffer.get_line ({:?}):'{:?}'",
//     //     start_line,
//     //     rslice2line(&line),
//     //   );
//     // } else {
//     //   debug!("buffer.get_line ({:?}):None", start_line);
//     // }
//
//     match buffer.get_lines_at(start_line) {
//       Some(mut buflines) => {
//         // The `start_line` is inside the buffer.
//         // Render the lines from `start_line` till the end of the buffer or the window widget.
//
//         // The first `row` (0) in the window maps to the `start_line` in the buffer.
//         let mut row = 0;
//
//         while row < height {
//           match buflines.next() {
//             Some(line) => {
//               // For the row in current window widget, if has the line in buffer.
//               let mut col = 0_u16;
//
//               for chunk in line.chunks() {
//                 if col >= width {
//                   break;
//                 }
//                 for ch in chunk.chars() {
//                   if col >= width {
//                     break;
//                   }
//                   if ch != '\n' {
//                     let cell = Cell::from(ch);
//                     let cell_upos = point!(x: col + upos.x(), y: row + upos.y());
//                     // debug!(
//                     //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
//                     //   row, col, ch, cell_upos
//                     // );
//                     canvas.frame_mut().set_cell(cell_upos, cell);
//                   }
//                   col += 1;
//                 }
//               }
//
//               // The line doesn't fill the whole row in current widget, fill left parts with empty
//               // cells.
//               if row < height && col < width - 1 {
//                 let cells_upos = point!(x: col + upos.x(), y: row + upos.y());
//                 let cells_len = (width - col) as usize;
//                 // debug!(
//                 //   "2-row:{:?}, col:{:?}, cells upos:{:?}, cells len:{:?}",
//                 //   row, col, cells_upos, cells_len,
//                 // );
//                 canvas
//                   .frame_mut()
//                   .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
//                   .unwrap();
//               }
//             }
//             None => {
//               // If there's no more lines in the buffer, simply set the whole line to empty for
//               // left parts of the window.
//               let cells_upos = point!(x: upos.x(), y: row + upos.y());
//               let cells_len = width as usize;
//               // debug!(
//               //   "3-row:{:?}, cells upos:{:?}, cells len:{:?}",
//               //   row, cells_upos, cells_len,
//               // );
//               canvas
//                 .frame_mut()
//                 .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
//                 .unwrap();
//             }
//           }
//           // Iterate to next row.
//           row += 1;
//         }
//       }
//       None => {
//         // The `start_line` is outside of the buffer.
//         // Render the whole window contents as empty cells.
//
//         // The first `row` (0) in the window maps to the `start_line` in the buffer.
//         let mut row = 0;
//
//         while row < height {
//           // There's no lines in the buffer, simply set the whole line to empty.
//           let cells_upos = point!(x: upos.x(), y: row + upos.y());
//           let cells_len = width as usize;
//           // debug!(
//           //   "4-row:{:?}, cells upos:{:?}, cells len:{:?}",
//           //   row, cells_upos, cells_len,
//           // );
//           canvas
//             .frame_mut()
//             .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
//             .unwrap();
//           row += 1;
//         }
//       }
//     }
//   }
//
//   /// Draw buffer from `end_line` in reverse order.
//   pub fn _draw_from_bottom(
//     &mut self,
//     _canvas: &mut Canvas,
//     _end_line: usize,
//     _start_column: usize,
//     _end_column: usize,
//   ) {
//     unimplemented!()
//   }
// }
// // Draw }

impl Widgetable for WindowContent {
  fn draw(&mut self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let upos: U16Pos = actual_shape.min().into();
    let height = actual_shape.height();
    let width = actual_shape.width();

    // If size is zero, exit.
    if height == 0 || width == 0 {
      return;
    }

    let viewport = self.viewport.clone().unwrap();
    let viewport = viewport.as_ref();

    // If viewport has no lines.
    if viewport.end_line() <= viewport.start_line() {
      return;
    }

    let buffer = self.buffer.upgrade().unwrap();
    let buffer = rlock!(buffer);

    let mut row_idx = 0_u16;
    let mut line_idx = viewport.start_line();
    let mut lines_slice = buffer.get_lines_at(line_idx).unwrap();

    let mut start_fills_count = 0_usize;
    let mut end_fills_count = 0_usize;

    while line_idx < viewport.end_line() {
      let line_slice = lines_slice.next().unwrap();
      let line_viewport = viewport.lines().get(&line_idx).unwrap();

      let row_viewport = &line_viewport.rows;
      let first_row = row_viewport.first_key_value().unwrap();
      let last_row = row_viewport.last_key_value().unwrap();
      let first_row_idx = *first_row.0;
      let last_row_idx = *last_row.0;

      for (r_idx, r) in row_viewport.iter() {
        debug_assert_eq!(*r_idx, row_idx);

        let mut col_idx = 0_u16;

        let start_fills = if row_idx == first_row_idx && line_viewport.start_filled_columns > 0 {
          start_fills_count += 1;
          assert!(start_fills_count == 1);
          line_viewport.start_filled_columns as u16
        } else {
          0_u16
        };
        let end_fills = if row_idx == last_row_idx && line_viewport.end_filled_columns > 0 {
          assert!(start_fills_count == 1);
          end_fills_count += 1;
          assert!(end_fills_count == 1);
          line_viewport.end_filled_columns as u16
        } else {
          0_u16
        };

        // Render start fills.
        if start_fills > 0 {
          let cells = std::iter::repeat_n('>', line_viewport.start_filled_columns)
            .map(Cell::from)
            .collect::<Vec<_>>();
          let cells_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
          canvas.frame_mut().set_cells_at(cells_upos, cells);
          col_idx += line_viewport.start_filled_columns as u16;
        }

        // Render line content.
        if r.end_char_idx > r.start_char_idx {
          let mut total_width = 0_usize;
          let mut char_idx = r.start_char_idx;
          let mut chars_slice = line_slice.get_chars_at(r.start_char_idx).unwrap();
          while char_idx < r.end_char_idx {
            let c = chars_slice.next().unwrap();
            let (unicode_symbol, unicode_width) = buffer.char_symbol(c);

            let cell = Cell::with_symbol(unicode_symbol);
            let cell_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
            canvas.frame_mut().set_cell(cell_upos, cell);

            col_idx += 1;
            char_idx += 1;
            total_width += unicode_width;
          }
          debug_assert_eq!(total_width, r.end_bcolumn - r.start_bcolumn);
        }

        // Render left empty parts.
        if width > (r.end_bcolumn - r.start_bcolumn) as u16 + start_fills + end_fills {
          let left_parts_length =
            width - ((r.end_bcolumn - r.start_bcolumn) as u16 + start_fills + end_fills);
          let cells = std::iter::repeat_n(' ', left_parts_length as usize)
            .map(Cell::from)
            .collect::<Vec<_>>();
          let cells_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
          canvas.frame_mut().set_cells_at(cells_upos, cells);
          col_idx += left_parts_length;
        }

        // Render end fills.
        if end_fills > 0 {
          let cells = std::iter::repeat_n('<', line_viewport.end_filled_columns)
            .map(Cell::from)
            .collect::<Vec<_>>();
          let cells_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
          canvas.frame_mut().set_cells_at(cells_upos, cells);

          col_idx += line_viewport.end_filled_columns as u16;
        }
        debug_assert_eq!(width, col_idx);

        row_idx += 1;
      }

      line_idx += 1;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::buf::BufferArc;
  use crate::cart::U16Size;
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use std::sync::Once;
  use tracing::info;

  #[allow(dead_code)]
  static INIT: Once = Once::new();

  fn make_buffer_from_file(filename: String) -> BufferArc {
    let rop: Rope = Rope::from_reader(BufReader::new(File::open(filename).unwrap())).unwrap();
    let buf: Buffer = Buffer::from(rop);
    Buffer::to_arc(buf)
  }

  fn make_buffer_from_lines(lines: Vec<&str>) -> BufferArc {
    let mut rop: RopeBuilder = RopeBuilder::new();
    for line in lines.iter() {
      rop.append(line);
    }
    let buf: Buffer = Buffer::from(rop);
    Buffer::to_arc(buf)
  }

  fn make_empty_buffer() -> BufferArc {
    let buf: Buffer = RopeBuilder::new().into();
    Buffer::to_arc(buf)
  }

  #[test]
  fn _draw_from_top_for_nowrap1() {
    // INIT.call_once(test_log_init);
    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);

    let terminal_size = U16Size::new(10, 10);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder().wrap(false).build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (10, 10));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(10, 10);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_nowrap(&mut canvas, 0, 0, 10);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    let expect = buffer
      .read()
      .lines()
      .take(10)
      .map(|l| l.as_str().unwrap().chars().take(10).collect::<String>())
      .collect::<Vec<_>>();
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 10);
    assert!(expect.len() <= 10);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 10);
      if i < expect.len() {
        let e = expect[i].clone();
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 10].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_nowrap2() {
    // INIT.call_once(test_log_init);
    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let terminal_size = U16Size::new(27, 15);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder().wrap(false).build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (27, 15));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(27, 15);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_nowrap(&mut canvas, 1, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    let expect = buffer
      .read()
      .lines()
      .skip(1)
      .take(15)
      .map(|l| l.as_str().unwrap().chars().take(27).collect::<String>())
      .collect::<Vec<_>>();
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 15);
    assert!(expect.len() <= 15);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 27);
      if i < expect.len() {
        let e = expect[i].clone();
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 27].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_nowrap3() {
    // INIT.call_once(test_log_init);
    let buffer = make_empty_buffer();
    let terminal_size = U16Size::new(20, 18);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder().wrap(false).build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (20, 18));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(20, 18);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_nowrap(&mut canvas, 0, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    assert_eq!(actual.len(), 18);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 20);
      info!("{:?} a:{:?}", i, a);
      assert!(a
        .chars()
        .filter(|c| *c != ' ')
        .collect::<Vec<_>>()
        .is_empty());
    }
  }

  #[test]
  fn _draw_from_top_for_nowrap4() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "This is a quite simple and smal",
      "But still it contains several t",
      "  1. When the line is small eno",
      "  2. When the line is too long ",
      "     * The extra parts are been",
      "     * The extra parts are spli",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
    ];

    let width = 31;
    let height = 19;
    let terminal_size = U16Size::new(width, height);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder().wrap(false).build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (width as isize, height as isize));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(width, height);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_nowrap(&mut canvas, 1, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), height as usize);
    assert_eq!(actual.len(), expect.len());
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == width as usize);
      if i < expect.len() {
        let e = expect[i];
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 27].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak1() {
    // INIT.call_once(test_log_init);
    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "Hello, RSV",
      "IM!       ",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.      ",
      "But still ",
      "it contain",
      "s several ",
    ];

    let terminal_size = U16Size::new(10, 10);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder().wrap(true).build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (10, 10));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(10, 10);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_wrap_nolinebreak(&mut canvas, 0, 0, 10);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 10);
    assert!(expect.len() <= 10);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 10);
      if i < expect.len() {
        let e = expect[i];
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 10].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak2() {
    // INIT.call_once(test_log_init);
    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "This is a quite simple and ",
      "small test lines.          ",
      "But still it contains sever",
      "al things we want to test: ",
      "  1. When the line is small",
      " enough to completely put i",
      "nside a row of the window c",
      "ontent widget, then the lin",
      "e-wrap and word-wrap doesn'",
      "t affect the rendering.    ",
      "  2. When the line is too l",
      "ong to be completely put in",
      " a row of the window conten",
      "t widget, there're multiple",
      " cases:                    ",
    ];
    let terminal_size = U16Size::new(27, 15);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder().wrap(true).build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (27, 15));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(27, 15);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_wrap_nolinebreak(&mut canvas, 1, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 15);
    assert!(expect.len() <= 15);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 27);
      if i < expect.len() {
        let e = expect[i];
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 27].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak3() {
    // INIT.call_once(test_log_init);
    let buffer = make_empty_buffer();

    let terminal_size = U16Size::new(20, 18);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder().wrap(true).build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (20, 18));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(20, 18);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_wrap_nolinebreak(&mut canvas, 0, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    assert_eq!(actual.len(), 18);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 20);
      info!("{:?} a:{:?}", i, a);
      assert!(a
        .chars()
        .filter(|c| *c != ' ')
        .collect::<Vec<_>>()
        .is_empty());
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak4() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "This is a quite sim",
      "ple and small test ",
      "lines.             ",
      "But still it contai",
      "ns several things w",
      "e want to test:    ",
      "  1. When the line ",
      "is small enough to ",
      "completely put insi",
      "de a row of the win",
      "dow content widget,",
      " then the line-wrap",
      " and word-wrap does",
      "n't affect the rend",
      "ering.             ",
      "  2. When the line ",
      "is too long to be c",
      "ompletely put in a ",
      "row of the window c",
      "ontent widget, ther",
      "e're multiple cases",
      ":                  ",
      "     * The extra pa",
      "rts are been trunca",
      "ted if both line-wr",
      "ap and word-wrap op",
    ];

    let width = 19;
    let height = 26;
    let terminal_size = U16Size::new(width, height);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder().wrap(true).build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (width as isize, height as isize));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(width, height);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_wrap_nolinebreak(&mut canvas, 1, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), height as usize);
    assert_eq!(actual.len(), expect.len());
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == width as usize);
      if i < expect.len() {
        let e = expect[i];
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 27].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_linebreak1() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "Hello,    ",
      "RSVIM!    ",
      "This is a ",
      "quite     ",
      "simple and",
      " small    ",
      "test lines",
      ".         ",
      "But still ",
      "it        ",
    ];

    let terminal_size = U16Size::new(10, 10);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (10, 10));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(10, 10);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_wrap_linebreak(&mut canvas, 0, 0, 10);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 10);
    assert!(expect.len() <= 10);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 10);
      if i < expect.len() {
        let e = expect[i];
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 10].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_linebreak2() {
    // INIT.call_once(test_log_init);
    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "This is a quite simple and ",
      "small test lines.          ",
      "But still it contains      ",
      "several things we want to  ",
      "test:                      ",
      "  1. When the line is small",
      " enough to completely put  ",
      "inside a row of the window ",
      "content widget, then the   ",
      "line-wrap and word-wrap    ",
      "doesn't affect the         ",
      "rendering.                 ",
      "  2. When the line is too  ",
      "long to be completely put  ",
      "in a row of the window     ",
    ];
    let terminal_size = U16Size::new(27, 15);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (27, 15));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(27, 15);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_wrap_linebreak(&mut canvas, 1, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), 15);
    assert!(expect.len() <= 15);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 27);
      if i < expect.len() {
        let e = expect[i];
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 27].join(""));
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_linebreak3() {
    // INIT.call_once(test_log_init);
    let buffer = make_empty_buffer();

    let terminal_size = U16Size::new(20, 18);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (20, 18));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(20, 18);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_wrap_linebreak(&mut canvas, 0, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    assert_eq!(actual.len(), 18);
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == 20);
      info!("{:?} a:{:?}", i, a);
      assert!(a
        .chars()
        .filter(|c| *c != ' ')
        .collect::<Vec<_>>()
        .is_empty());
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_linebreak4() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "This is a    ",
      "quite simple ",
      "and small    ",
      "test lines.  ",
      "But still it ",
      "contains     ",
      "several      ",
      "things we    ",
      "want to test:",
      "             ",
      "  1. When the",
      " line is     ",
      "small enough ",
      "to completely",
      " put inside a",
      " row of the  ",
      "window       ",
      "content      ",
      "widget, then ",
      "the line-wrap",
      " and word-   ",
      "wrap doesn't ",
      "affect the   ",
      "rendering.   ",
      "  2. When the",
      " line is too ",
      "long to be   ",
      "completely   ",
      "put in a row ",
      "of the window",
      " content     ",
    ];
    let width = 13;
    let height = 31;
    let terminal_size = U16Size::new(width, height);
    let mut tree = Tree::new(terminal_size);
    let window_options = WindowLocalOptions::builder()
      .wrap(true)
      .line_break(true)
      .build();
    tree.set_local_options(&window_options);
    let window_content_shape = IRect::new((0, 0), (width as isize, height as isize));
    let mut window_content =
      WindowContent::new(window_content_shape, Arc::downgrade(&buffer), &mut tree);
    let canvas_size = U16Size::new(width, height);
    let mut canvas = Canvas::new(canvas_size);
    window_content._draw_from_top_for_wrap_linebreak(&mut canvas, 1, 0, 0);
    let actual = canvas
      .frame()
      .raw_symbols_with_placeholder(" ".to_compact_string())
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{:?}", actual);
    info!("expect:{:?}", expect);
    assert_eq!(actual.len(), height as usize);
    assert_eq!(actual.len(), expect.len());
    for (i, a) in actual.into_iter().enumerate() {
      assert!(a.len() == width as usize);
      if i < expect.len() {
        let e = expect[i];
        info!("{:?} a:{:?}, e:{:?}", i, a, e);
        assert!(a.len() == e.len() || e.is_empty());
        if a.len() == e.len() {
          assert_eq!(a, e);
        }
      } else {
        info!("{:?} a:{:?}, e:empty", i, a);
        assert_eq!(a, [" "; 27].join(""));
      }
    }
  }
}
