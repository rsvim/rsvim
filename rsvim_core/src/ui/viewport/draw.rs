//! Draw a text (with its viewport) on a canvas (with its actual shape).

use crate::buf::text::Text;
use crate::hl::ColorScheme;
use crate::hl::Highlight;
use crate::prelude::*;
use crate::syntax::Syntax;
use crate::syntax::SyntaxCapturePoint;
use crate::syntax::SyntaxCaptureValue;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::Cell;
use crate::ui::viewport::Viewport;
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use crossterm::style::Attributes;
use std::convert::From;

/// Draw a text (with its viewport) on a canvas (with its actual shape).
pub fn draw(
  viewport: &Viewport,
  text: &Text,
  syntax: &Option<Syntax>,
  colorscheme: &Option<ColorScheme>,
  actual_shape: &U16Rect,
  canvas: &mut Canvas,
) {
  // If size is zero, exit.
  if actual_shape.size().is_zero() {
    trace!("Draw viewport, actual shape is zero");
    return;
  }

  let upos: U16Pos = actual_shape.min().into();
  let height = actual_shape.height();
  let width = actual_shape.width();

  let mut row_idx = 0_u16;
  let mut line_idx = viewport.start_line_idx();

  let mut buflines = text.rope().lines_at(line_idx);

  let mut last_colorscheme_hl: Option<Highlight> = None;
  let mut last_hl_capture: Option<SyntaxCaptureValue> = None;

  // Try to allocate only once for each draw.
  let bump = Bump::with_capacity((height as usize) * (width as usize));

  let set_bg = |cell: &mut Cell| {
    if let Some(colorscheme) = colorscheme {
      cell.set_fg(colorscheme.ui_foreground());
      cell.set_bg(colorscheme.ui_background());
      cell.set_attrs(Attributes::none());
    }
  };

  // If viewport is empty (i.e. no lines), it skips this part.
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
          let mut cells: BumpVec<Cell> = BumpVec::new_in(&bump);
          std::iter::repeat_n('>', start_fills as usize).for_each(|ch| {
            let mut cell = Cell::from(ch);
            set_bg(&mut cell);
            cells.push(cell);
          });

          let cells_upos = point!(col_idx + upos.x(), row_idx + upos.y());
          canvas
            .frame_mut()
            .set_cells_at(cells_upos, cells.clone().into_iter());
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

            // The canvas system is designed with a `M x N` logic cells in the
            // beginning, I was thinking each cell should render a 1-width char
            // in the terminal.
            // Then I noticed that, an ASCII control code, ASCII char or a
            // unicode char can use different width (cells) in the terminal:
            //
            // - Use 0-width, for example LF/CR, but actually a widget will
            //   never render a 0-width char to canvas.
            // - Use 1-width, most ASCII chars and some special characters.
            //   This is the happy case.
            // - Use 2-width, most CJK chars and some special unicodes.
            // - Use more than 2-width, for example '\t' (Tab) by default uses
            //   8-width.
            //
            // But in the canvas system, characters are managed by logic "cell"
            // (a logic cell is a "CompactString"). If a logic cell actually
            // displays multiple "term cells" in the terminal, then the whole
            // row in the terminal can be overflow.
            //
            // Thus we need to work around this case, by setting the following
            // cells to empty string `""`.
            //
            // For example now we have 2 cases:
            //
            // - `\t` (Tab), (by default) it is 8-width. We create 8 logic
            //   cells, the 1st cell is the `\t` (Tab) char, the following 7
            //   cells are `""` empty string.
            // - `好` (CJK), it is 2-width. We create 2 cells, the 1st cell is
            //   the `好` char, the following 1 cell is `""` empty string.

            if unicode_width > 0 {
              let cap_point = SyntaxCapturePoint { line_idx, char_idx };

              if let Some(syntax) = syntax
                && let Some(syn_highlight_capture) = syntax.highlight_capture()
                && let Some(colorscheme) = colorscheme
                && syn_highlight_capture
                  .as_ref()
                  .nodes()
                  .contains_key(&cap_point)
              {
                let hl_caps = syn_highlight_capture
                  .as_ref()
                  .nodes()
                  .get(&cap_point)
                  .unwrap();
                trace!("captured highlight, {:?}:{:?}", cap_point, hl_caps);
                for (i_cap, hl_cap) in hl_caps.values.iter().enumerate() {
                  if let Some(hl) = colorscheme.highlights().get(&hl_cap.name) {
                    trace!(
                      "resolved highlight-[{}], captured:{:?}, resolved:{:?}",
                      i_cap, hl_cap, hl
                    );
                    let fg = colorscheme.resolve_fg(&hl.fg);
                    let bg = colorscheme.resolve_bg(&hl.bg);
                    last_colorscheme_hl = Some(Highlight {
                      fg: Some(fg),
                      bg: Some(bg),
                      attrs: hl.attrs,
                    });
                    last_hl_capture = Some(hl_cap.clone());
                    break;
                  }
                }
              }

              let set_hl = |cell: &mut Cell| {
                if let Some(colorscheme_hl) = last_colorscheme_hl
                  && let Some(ref hl_capture) = last_hl_capture
                  && cap_point >= hl_capture.range.start_point
                  && cap_point < hl_capture.range.end_point
                {
                  cell.set_fg(colorscheme_hl.fg.unwrap());
                  cell.set_bg(colorscheme_hl.bg.unwrap());
                  cell.set_attrs(colorscheme_hl.attrs);
                } else if let Some(colorscheme) = colorscheme {
                  cell.set_fg(colorscheme.ui_foreground());
                  cell.set_bg(colorscheme.ui_background());
                  cell.set_attrs(Attributes::none());
                }
                trace!("set_hl:{:?}", cell);
              };

              let cell_upos = point!(col_idx + upos.x(), row_idx + upos.y());
              if unicode_width > 1 {
                let mut cells: BumpVec<Cell> = BumpVec::new_in(&bump);

                // Unicode width > 1
                let mut cell = Cell::with_symbol(unicode_symbol);
                set_hl(&mut cell);

                cells.push(cell);
                for _i in 0..(unicode_width - 1) {
                  let mut cell = Cell::empty();
                  set_hl(&mut cell);
                  cells.push(cell);
                }
                canvas
                  .frame_mut()
                  .set_cells_at(cell_upos, cells.clone().into_iter());
              } else {
                let mut cell = Cell::with_symbol(unicode_symbol);
                set_hl(&mut cell);

                // Unicode width = 1
                canvas.frame_mut().set_cell(cell_upos, cell);
              };

              col_idx += unicode_width as u16;
            }

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
          let mut cells: BumpVec<Cell> = BumpVec::new_in(&bump);

          let left_length = width - occupied_length;
          std::iter::repeat_n(' ', left_length as usize).for_each(|ch| {
            let mut cell = Cell::from(ch);
            set_bg(&mut cell);
            cells.push(cell);
          });

          let cells_upos = point!(col_idx + upos.x(), row_idx + upos.y());
          canvas
            .frame_mut()
            .set_cells_at(cells_upos, cells.clone().into_iter());
          col_idx += left_length;
        }

        // Render end fills.
        if end_fills > 0 {
          let mut cells: BumpVec<Cell> = BumpVec::new_in(&bump);

          std::iter::repeat_n('<', end_fills as usize).for_each(|ch| {
            let mut cell = Cell::from(ch);
            set_bg(&mut cell);
            cells.push(cell);
          });

          let cells_upos = point!(col_idx + upos.x(), row_idx + upos.y());
          canvas
            .frame_mut()
            .set_cells_at(cells_upos, cells.clone().into_iter());

          col_idx += end_fills;
        }
        debug_assert_eq!(width, col_idx);

        row_idx += 1;
      }
    }

    line_idx += 1;
  }

  // If buffer has no more lines, or even the buffer/viewport is empty. Render
  // empty spaces to left parts of the window content.
  //
  // NOTE: If the viewport is empty (i.e. it has no lines), it goes to this
  // part as well.
  while row_idx < height {
    let mut cells: BumpVec<Cell> = BumpVec::new_in(&bump);

    std::iter::repeat_n(' ', width as usize).for_each(|ch| {
      let mut cell = Cell::from(ch);
      set_bg(&mut cell);
      cells.push(cell);
    });

    let cells_upos = point!(upos.x(), row_idx + upos.y());
    canvas
      .frame_mut()
      .set_cells_at(cells_upos, cells.clone().into_iter());
    row_idx += 1;
  }
}
