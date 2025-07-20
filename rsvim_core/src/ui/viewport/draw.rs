//! Draw a text (with its viewport) on a canvas (with its actual shape).

use crate::buf::text::Text;
use crate::prelude::*;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::viewport::Viewport;

use geo::point;
use std::convert::From;
use tracing::trace;

/// Draw a text (with its viewport) on a canvas (with its actual shape).
pub fn draw(
  viewport: &Viewport,
  text: &Text,
  actual_shape: &U16Rect,
  canvas: &mut Canvas,
) {
  let upos: U16Pos = actual_shape.min().into();
  let height = actual_shape.height();
  let width = actual_shape.width();

  // If size is zero, exit.
  if height == 0 || width == 0 {
    trace!("Draw viewport, actual shape is zero");
    return;
  }

  // If viewport has no lines.
  if viewport.end_line_idx() <= viewport.start_line_idx() {
    trace!("Draw viewport, viewport is empty");
    return;
  }

  let mut row_idx = 0_u16;
  let mut line_idx = viewport.start_line_idx();

  let mut buflines = text.rope().lines_at(line_idx);

  while line_idx < viewport.end_line_idx() {
    debug_assert!(row_idx < height);

    let mut start_fills_count = 0_usize;
    let mut end_fills_count = 0_usize;

    let bline = buflines.next().unwrap();
    let line_viewport = viewport.lines().get(&line_idx).unwrap();

    let rows_viewport = line_viewport.rows();

    if !rows_viewport.is_empty() {
      let first_row = rows_viewport.first().unwrap();
      let last_row = rows_viewport.last().unwrap();
      let first_row_idx = *first_row.0;
      let last_row_idx = *last_row.0;

      for (r_idx, r_viewport) in rows_viewport.iter() {
        debug_assert_eq!(*r_idx, row_idx);
        debug_assert!(row_idx < height);

        let mut col_idx = 0_u16;

        let start_fills = if row_idx == first_row_idx
          && line_viewport.start_filled_cols() > 0
        {
          start_fills_count += 1;
          debug_assert_eq!(start_fills_count, 1);
          line_viewport.start_filled_cols() as u16
        } else {
          0_u16
        };
        let end_fills =
          if row_idx == last_row_idx && line_viewport.end_filled_cols() > 0 {
            end_fills_count += 1;
            debug_assert_eq!(end_fills_count, 1);
            line_viewport.end_filled_cols() as u16
          } else {
            0_u16
          };

        // Render start fills.
        if start_fills > 0 {
          let cells = std::iter::repeat_n('>', start_fills as usize)
            .map(Cell::from)
            .collect::<Vec<_>>();
          let cells_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
          canvas.frame_mut().set_cells_at(cells_upos, cells);
          col_idx += start_fills;
        }

        // Render line content.
        if r_viewport.end_char_idx() > r_viewport.start_char_idx() {
          // let mut total_width = 0_usize;
          let mut char_idx = r_viewport.start_char_idx();
          let mut chars_iter =
            bline.get_chars_at(r_viewport.start_char_idx()).unwrap();
          while char_idx < r_viewport.end_char_idx() {
            let c = chars_iter.next().unwrap();
            let (unicode_symbol, unicode_width) = text.char_symbol_and_width(c);

            let cell = Cell::with_symbol(unicode_symbol);
            let cell_upos =
              point!(x: col_idx + upos.x(), y: row_idx + upos.y());
            canvas.frame_mut().set_cell(cell_upos, cell);

            // FIXME: The canvas system is designed with a `M x N` cells.
            // Ideally each cell should render a 1-width char in terminal.
            // But in real-world, if a unicode char is 2-width, it will
            // actually uses 2 cells in terminal, but only 1 cell in our
            // canvas.
            // Thus we need to work around this case, by setting the
            // following cells to empty string `""`.
            //
            // For example we have below 2 cases:
            //
            // - `\t` (Tab), (by default) it is 8-width. We will create 8
            // cells (because in canvas, 1 cell is for 1-width display),
            // the 1st cell is the `\t` (Tab) char, the following 7 cells
            // are `""` empty string.
            // - `好`, it is 2-width. We will create 2 cells, the 1st cell
            // is the `好` char, the following 1 cell is `""` empty string.

            let cell_upos =
              point!(x: col_idx + upos.x(), y: row_idx + upos.y());
            canvas.frame_mut().set_cell(cell_upos, cell);

            col_idx += unicode_width as u16;
            char_idx += 1;
          }
        }

        // Render left empty parts.
        let end_dcol_idx =
          text.width_before(line_idx, r_viewport.end_char_idx());
        let start_dcol_idx =
          text.width_before(line_idx, r_viewport.start_char_idx());
        let occupied_length =
          (end_dcol_idx - start_dcol_idx) as u16 + start_fills + end_fills;

        if width > occupied_length {
          let left_length = width - occupied_length;
          let cells = std::iter::repeat_n(' ', left_length as usize)
            .map(Cell::from)
            .collect::<Vec<_>>();
          let cells_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
          canvas.frame_mut().set_cells_at(cells_upos, cells);
          col_idx += left_length;
        }

        // Render end fills.
        if end_fills > 0 {
          let cells = std::iter::repeat_n('<', end_fills as usize)
            .map(Cell::from)
            .collect::<Vec<_>>();
          let cells_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
          canvas.frame_mut().set_cells_at(cells_upos, cells);

          col_idx += end_fills;
        }
        debug_assert_eq!(width, col_idx);

        row_idx += 1;
      }
    }

    line_idx += 1;
  }

  // If buffer has no more lines, render empty spaces to left parts of the window content.
  while row_idx < height {
    let cells = std::iter::repeat_n(' ', width as usize)
      .map(Cell::from)
      .collect::<Vec<_>>();
    let cells_upos = point!(x: upos.x(), y: row_idx + upos.y());
    canvas.frame_mut().set_cells_at(cells_upos, cells);
    row_idx += 1;
  }
}
