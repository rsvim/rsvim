//! Vim cmdline.

#![allow(dead_code)]

use crate::content::TemporaryContentsWk;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::WindowLocalOptions;
use crate::{inode_impl, lock};

#[derive(Debug, Clone)]
/// The Vim cmdline.
pub struct Cmdline {
  base: InodeBase,

  // Cmdline content temporary content.
  contents: TemporaryContentsWk,

  // Cmdline content viewport.
  viewport: ViewportWk,

  options: WindowLocalOptions,
}

impl Cmdline {
  pub fn new(opts: &WindowLocalOptions, shape: IRect, contents: TemporaryContentsWk) -> Self {
    let options = *opts;
    let base = InodeBase::new(shape);
    Self {
      base,
      contents,
      options,
    }
  }
}

inode_impl!(Cmdline, base);

impl Widgetable for Cmdline {
  fn draw(&self, canvas: &mut crate::ui::canvas::Canvas) {
    let actual_shape = self.actual_shape();
    let upos: U16Pos = actual_shape.min().into();
    let height = actual_shape.height();
    let width = actual_shape.width();

    // If size is zero, exit.
    if height == 0 || width == 0 {
      trace!("Draw cmdline, actual shape is zero");
      return;
    }

    let viewport = self.viewport.upgrade().unwrap();

    // If viewport has no lines.
    if viewport.end_line_idx() <= viewport.start_line_idx() {
      trace!("Draw window content, viewport is empty");
      return;
    }

    // trace!(
    //   "Draw window content, actual shape min pos:{:?}, height/width:{}/{}",
    //   upos,
    //   height,
    //   width
    // );
    // trace!("Draw window content, viewport:{:?}", viewport);

    let buffer = self.buffer.upgrade().unwrap();
    let buffer = lock!(buffer);

    let mut row_idx = 0_u16;
    let mut line_idx = viewport.start_line_idx();

    let mut buflines = buffer.text().rope().lines_at(line_idx);

    while line_idx < viewport.end_line_idx() {
      debug_assert!(row_idx < height);

      let mut start_fills_count = 0_usize;
      let mut end_fills_count = 0_usize;

      let bline = buflines.next().unwrap();
      let line_viewport = viewport.lines().get(&line_idx).unwrap();

      // trace!(
      //   "0-line_idx:{}, row_idx:{}, line_viewport:{:?}",
      //   line_idx,
      //   row_idx,
      //   line_viewport
      // );

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

          let start_fills = if row_idx == first_row_idx && line_viewport.start_filled_cols() > 0 {
            start_fills_count += 1;
            debug_assert_eq!(start_fills_count, 1);
            line_viewport.start_filled_cols() as u16
          } else {
            0_u16
          };
          let end_fills = if row_idx == last_row_idx && line_viewport.end_filled_cols() > 0 {
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
            // trace!(
            //   "1-line_idx:{}, row_idx:{}, col_idx:{}, line_viewport:{:?}, r:{:?}",
            //   line_idx,
            //   row_idx,
            //   col_idx,
            //   line_viewport,
            //   r_viewport
            // );
          }

          // Render line content.
          if r_viewport.end_char_idx() > r_viewport.start_char_idx() {
            // let mut total_width = 0_usize;
            let mut char_idx = r_viewport.start_char_idx();
            let mut chars_iter = bline.get_chars_at(r_viewport.start_char_idx()).unwrap();
            while char_idx < r_viewport.end_char_idx() {
              let c = chars_iter.next().unwrap();
              let (unicode_symbol, unicode_width) = buffer.text().char_symbol(c);

              let cell = Cell::with_symbol(unicode_symbol);
              let cell_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
              canvas.frame_mut().set_cell(cell_upos, cell);

              col_idx += unicode_width as u16;
              char_idx += 1;
              // total_width += unicode_width;
            }
            // trace!(
            //   "2-line_idx:{}, row_idx:{}, col_idx:{}, total_width:{}, line_viewport:{:?}, r:{:?}",
            //   line_idx,
            //   row_idx,
            //   col_idx,
            //   total_width,
            //   line_viewport,
            //   r_viewport
            // );
          }

          // Render left empty parts.
          let end_dcol_idx = buffer
            .text()
            .width_before(line_idx, r_viewport.end_char_idx());
          let start_dcol_idx = buffer
            .text()
            .width_before(line_idx, r_viewport.start_char_idx());
          let occupied_length = (end_dcol_idx - start_dcol_idx) as u16 + start_fills + end_fills;

          if width > occupied_length {
            let left_length = width - occupied_length;
            let cells = std::iter::repeat_n(' ', left_length as usize)
              .map(Cell::from)
              .collect::<Vec<_>>();
            let cells_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
            canvas.frame_mut().set_cells_at(cells_upos, cells);
            col_idx += left_length;
            // trace!(
            //   "3-line_idx:{}, row_idx:{}, col_idx:{}, left_length:{}, line_viewport:{:?}, r:{:?}",
            //   line_idx,
            //   row_idx,
            //   col_idx,
            //   left_length,
            //   line_viewport,
            //   r_viewport
            // );
          }

          // Render end fills.
          if end_fills > 0 {
            let cells = std::iter::repeat_n('<', end_fills as usize)
              .map(Cell::from)
              .collect::<Vec<_>>();
            let cells_upos = point!(x: col_idx + upos.x(), y: row_idx + upos.y());
            canvas.frame_mut().set_cells_at(cells_upos, cells);

            col_idx += end_fills;
            // trace!(
            //   "4-line_idx:{}, row_idx:{}, col_idx:{}, line_viewport:{:?}, r:{:?}",
            //   line_idx,
            //   row_idx,
            //   col_idx,
            //   line_viewport,
            //   r_viewport
            // );
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
}
