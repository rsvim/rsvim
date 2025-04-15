//! Vim window's text content widget.

use crate::buf::{Buffer, BufferWk};
use crate::prelude::*;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::ViewportWk;
use crate::{inode_impl, rlock, wlock};

use geo::point;
use std::convert::From;
use std::ptr::NonNull;
use tracing::trace;

#[derive(Debug, Clone)]
/// The widget contains text contents for Vim window.
pub struct WindowContent {
  base: InodeBase,

  // Buffer.
  buffer: BufferWk,

  // Viewport.
  viewport: ViewportWk,
}

impl WindowContent {
  /// Make window content.
  pub fn new(shape: IRect, buffer: BufferWk, viewport: ViewportWk) -> Self {
    let base = InodeBase::new(shape);
    WindowContent {
      base,
      buffer,
      viewport,
    }
  }
}

inode_impl!(WindowContent, base);

impl Widgetable for WindowContent {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let upos: U16Pos = actual_shape.min().into();
    let height = actual_shape.height();
    let width = actual_shape.width();

    // If size is zero, exit.
    if height == 0 || width == 0 {
      trace!("Draw window content, actual shape is zero");
      return;
    }

    let viewport = self.viewport.upgrade().unwrap();
    let viewport = rlock!(viewport);

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
    let mut buffer = wlock!(buffer);

    let mut row_idx = 0_u16;
    let mut line_idx = viewport.start_line_idx();

    unsafe {
      // Fix mutable borrow on `buffer`.
      let mut raw_buffer = NonNull::new(&mut *buffer as *mut Buffer).unwrap();

      let mut buflines = raw_buffer
        .as_ref()
        .get_rope()
        .get_lines_at(line_idx)
        .unwrap();

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
          let first_row = rows_viewport.first_key_value().unwrap();
          let last_row = rows_viewport.last_key_value().unwrap();
          let first_row_idx = *first_row.0;
          let last_row_idx = *last_row.0;

          for (r_idx, r_viewport) in rows_viewport.iter() {
            debug_assert_eq!(*r_idx, row_idx);
            debug_assert!(row_idx < height);

            let mut col_idx = 0_u16;

            let start_fills = if row_idx == first_row_idx && line_viewport.start_filled_cols() > 0 {
              start_fills_count += 1;
              assert!(start_fills_count == 1);
              line_viewport.start_filled_cols() as u16
            } else {
              0_u16
            };
            let end_fills = if row_idx == last_row_idx && line_viewport.end_filled_cols() > 0 {
              end_fills_count += 1;
              assert!(end_fills_count == 1);
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
                let (unicode_symbol, unicode_width) = raw_buffer.as_ref().char_symbol(c);

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
            let end_dcol_idx = raw_buffer
              .as_mut()
              .width_before(line_idx, r_viewport.end_char_idx());
            let start_dcol_idx = raw_buffer
              .as_mut()
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

// spellchecker:off
#[allow(unused_imports)]
#[cfg(test)]
mod tests {
  use super::*;

  use crate::buf::{BufferArc, BufferLocalOptions, BufferLocalOptionsBuilder};
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Tree;
  use crate::ui::widget::window::{
    Viewport, ViewportArc, ViewportOptions, WindowLocalOptions, WindowLocalOptionsBuilder,
  };

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use tracing::info;

  fn make_viewport(
    terminal_size: U16Size,
    window_options: WindowLocalOptions,
    buffer: BufferArc,
  ) -> ViewportArc {
    let mut tree = Tree::new(terminal_size);
    tree.set_global_local_options(&window_options);
    let actual_shape = U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height()));
    let viewport = {
      let mut buffer = wlock!(buffer);
      Viewport::from_top_left(&mut buffer, &actual_shape, &window_options, 0, 0)
    };
    Viewport::to_arc(viewport)
  }

  fn make_canvas(
    terminal_size: U16Size,
    window_options: WindowLocalOptions,
    buffer: BufferArc,
    viewport: ViewportArc,
  ) -> Canvas {
    let mut tree = Tree::new(terminal_size);
    tree.set_global_local_options(&window_options);
    let shape = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    let window_content =
      WindowContent::new(shape, Arc::downgrade(&buffer), Arc::downgrade(&viewport));
    let mut canvas = Canvas::new(terminal_size);
    window_content.draw(&mut canvas);
    canvas
  }

  #[allow(clippy::too_many_arguments)]
  fn assert_from_top_left(actual: &Canvas, expect: &[&str]) {
    let actual = actual
      .frame()
      .raw_symbols()
      .iter()
      .map(|cs| cs.join(""))
      .collect::<Vec<_>>();
    info!("actual:{}", actual.len());
    for a in actual.iter() {
      info!("{:?}", a);
    }
    info!("expect:{}", expect.len());
    for e in expect.iter() {
      info!("{:?}", e);
    }

    assert_eq!(actual.len(), expect.len());
    for i in 0..actual.len() {
      let e = &expect[i];
      let a = &actual[i];
      info!("i-{}, actual[{}]:{:?}, expect[{}]:{:?}", i, i, a, i, e);
      assert_eq!(e.len(), a.len());
      assert_eq!(e, a);
    }
  }

  #[test]
  fn draw_new_nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSV",
      "This is a ",
      "But still ",
      "  1. When ",
      "  2. When ",
      "     * The",
      "     * The",
      "          ",
      "          ",
      "          ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_nowrap2() {
    test_log_init();

    let terminal_size = U16Size::new(35, 6);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!                      ",
      "This is a quite simple and small te",
      "But still it contains several thing",
      "  1. When the line is small enough ",
      "  2. When the line is too long to b",
      "     * The extra parts are been tru",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_nowrap3() {
    test_log_init();

    let terminal_size = U16Size::new(33, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello,  R\tS\tV\tI\tM!\n",
        "这是一个非常简单而且非常短的测试例子，只包含几行文本内容。\n",
        "But still\tit\tcontains\tseveral things we want to test:\n",
        "  第一，当一行文本内容足够短，以至于能够被完全的放入一个窗口中时，then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "Hello,  R        S        V<<<<<<",
      "这是一个非常简单而且非常短的测试<",
      "But still        it        contai",
      "  第一，当一行文本内容足够短，以<",
      "  2. When the line is too long to",
      "     * The extra parts are been t",
      "     * The extra parts are split ",
      "                                 ",
      "                                 ",
      "                                 ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_nowrap4() {
    test_log_init();

    let terminal_size = U16Size::new(31, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!                  ",
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

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_nowrap5() {
    test_log_init();

    let terminal_size = U16Size::new(31, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();

    let buffer = make_empty_buffer(terminal_size.height(), buf_opts);
    let expect = vec![
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
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
      "                               ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_sync_nowrap1() {
    test_log_init();

    let terminal_size = U16Size::new(21, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!        ",
      "This is a quite simpl",
      "But still it contains",
      "  1. When the line is",
      "  2. When the line is",
      "     * The extra part",
      "     * The extra part",
      "                     ",
      "                     ",
      "                     ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport.clone());
    assert_from_top_left(&actual, &expect);

    let expect = vec![
      "  2. When the line is",
      "     * The extra part",
      "     * The extra part",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
      "                     ",
    ];
    let viewport = {
      let mut buffer = wlock!(buffer);
      let actual_shape = U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height()));
      let viewport = Viewport::from_top_left(&mut buffer, &actual_shape, &win_opts, 4, 0);
      Viewport::to_arc(viewport)
    };
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport.clone());
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_nolinebreak1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_nolinebreak2() {
    test_log_init();

    let terminal_size = U16Size::new(27, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "\t\t\t* The extra parts are split\tinto the next row,\tif either line-wrap\tor word-wrap options are been set. If the extra parts are still too long to\t来放在下一个横行内，一遍又一遍的重复这样的操作。This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "                        * T",
      "he extra parts are split   ",
      "        into the next row, ",
      "        if either line-wrap",
      "        or word-wrap option",
      "s are been set. If the extr",
      "a parts are still too long ",
      "to        来放在下一个横行 ",
      "内，一遍又一遍的重复这样的 ",
      "操作。This operation also e",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_nolinebreak3() {
    test_log_init();

    let terminal_size = U16Size::new(20, 9);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .build()
      .unwrap();

    let buffer = make_empty_buffer(terminal_size.height(), buf_opts);
    let expect = vec![
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_nolinebreak4() {
    test_log_init();

    let terminal_size = U16Size::new(19, 30);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things\t我们想要测试的：\n",
        "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
        "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!      ",
      "This is a quite sim",
      "ple and small test ",
      "lines.             ",
      "But still it contai",
      "ns several things  ",
      "        我们想要测 ",
      "试的：             ",
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
      "p doesn't affect th",
      "e rendering.       ",
      "        2. When the",
      " line is too long t",
      "o be completely put",
      " in a row of the wi",
      "ndow content widget",
      ", there're multiple",
      " cases:            ",
      "                *  ",
      "如果行换行和单词换 ",
      "行这两个选项都没有 ",
      "选中，那么这些超出 ",
      "窗口的文本内容会被 ",
      "截断。             ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_nolinebreak5() {
    test_log_init();

    let terminal_size = U16Size::new(19, 27);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things\t我们想要测试的：\n",
        "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
        "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!      ",
      "This is a quite sim",
      "ple and small test ",
      "lines.             ",
      "But still it contai",
      "ns several things  ",
      "        我们想要测 ",
      "试的：             ",
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
      "p doesn't affect th",
      "e rendering.       ",
      "        2. When the",
      " line is too long t",
      "o be completely put",
      " in a row of the wi",
      "ndow content widget",
      ", there're multiple",
      " cases:            ",
      "                *  ",
      "如果行换行和单词换 ",
      "行这两个选项都没有<",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_sync_wrap_nolinebreak1() {
    test_log_init();

    let terminal_size = U16Size::new(19, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(false)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things\t我们想要测试的：\n",
        "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "\t\t* 如果行换行和单词换行这两个选项都没有选中，那么这些超出窗口的文本内容会被截断。\n",
        "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!      ",
      "This is a quite sim",
      "ple and small test ",
      "lines.             ",
      "But still it contai",
      "ns several things  ",
      "        我们想要测 ",
      "试的：             ",
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport.clone());
    assert_from_top_left(&actual, &expect);

    let expect = vec![
      "        1. When the",
      " line is small enou",
      "gh to completely pu",
      "t inside a row of t",
      "he window content w",
      "idget, then the lin",
      "e-wrap and word-wra",
      "p doesn't affect th",
      "e rendering.       ",
      "        2. When the",
      " line is too long t",
      "o be completely put",
      " in a row of the wi",
      "ndow content widget",
      ", there're multiple",
    ];
    let viewport = {
      let mut buffer = wlock!(buffer);
      let actual_shape = U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height()));
      let viewport = Viewport::from_top_left(&mut buffer, &actual_shape, &win_opts, 3, 0);
      Viewport::to_arc(viewport)
    };
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_linebreak1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_linebreak2() {
    test_log_init();

    let terminal_size = U16Size::new(27, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!              ",
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
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_linebreak3() {
    test_log_init();

    let terminal_size = U16Size::new(20, 8);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_empty_buffer(terminal_size.height(), buf_opts);
    let expect = vec![
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
      "                    ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_linebreak4() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, 那么行换行和单词换行选项都不会影响最终的渲染效果。\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!",
      "             ",
      "This is a    ",
      "quite simple ",
      "andsmalltestl",
      "ineswithoutev",
      "enanewlinebre",
      "akbecausewewa",
      "nttotesthowit",
      "willhappensif",
      "thereisaveryl",
      "ongwordthatca",
      "nnotbeenpplac",
      "einsidearowof",
      "thewindowcont",
      "ent.         ",
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
      "widget, 那么 ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_new_wrap_linebreak5() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contai\tseveral things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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
      "it contai ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport);
    assert_from_top_left(&actual, &expect);
  }

  #[test]
  fn draw_sync_wrap_linebreak1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .build()
      .unwrap();

    let buffer = make_buffer_from_lines(
      terminal_size.height(),
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contai\tseveral things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
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
      "it contai ",
    ];

    let viewport = make_viewport(terminal_size, win_opts, buffer.clone());
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport.clone());
    assert_from_top_left(&actual, &expect);

    let expect = vec![
      "But still ",
      "it contai ",
      "          ",
      "several   ",
      "things we ",
      "want to   ",
      "test:     ",
      "  1. When ",
      "the line  ",
      "is small  ",
    ];

    let viewport = {
      let mut buffer = wlock!(buffer);
      let actual_shape = U16Rect::new((0, 0), (terminal_size.width(), terminal_size.height()));
      let viewport = Viewport::from_top_left(&mut buffer, &actual_shape, &win_opts, 2, 0);
      Viewport::to_arc(viewport)
    };
    let actual = make_canvas(terminal_size, win_opts, buffer.clone(), viewport.clone());
    assert_from_top_left(&actual, &expect);
  }
}
// spellchecker:on
