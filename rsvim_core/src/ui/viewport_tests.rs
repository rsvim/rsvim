#![allow(unused_imports)]

use super::viewport::*;

use crate::buf::BufferArc;
use crate::buf::opt::{BufferLocalOptions, BufferLocalOptionsBuilder, FileFormatOption};
use crate::prelude::*;
use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
use crate::test::log::init as test_log_init;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::{Window, WindowLocalOptions, WindowLocalOptionsBuilder};

use compact_str::ToCompactString;
use ropey::{Rope, RopeBuilder};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::rc::Rc;
use std::sync::{Arc, Once};
use tracing::info;

pub fn make_nowrap() -> WindowLocalOptions {
  WindowLocalOptionsBuilder::default()
    .wrap(false)
    .build()
    .unwrap()
}

pub fn make_wrap_nolinebreak() -> WindowLocalOptions {
  WindowLocalOptionsBuilder::default().build().unwrap()
}

pub fn make_wrap_linebreak() -> WindowLocalOptions {
  WindowLocalOptionsBuilder::default()
    .line_break(true)
    .build()
    .unwrap()
}

pub fn make_window(
  terminal_size: U16Size,
  buffer: BufferArc,
  window_options: &WindowLocalOptions,
) -> Window {
  let mut tree = Tree::new(terminal_size);
  tree.set_global_local_options(window_options);
  let window_shape = IRect::new(
    (0, 0),
    (
      terminal_size.width() as isize,
      terminal_size.height() as isize,
    ),
  );
  Window::new(
    tree.global_local_options(),
    window_shape,
    Arc::downgrade(&buffer),
  )
}

#[allow(clippy::too_many_arguments)]
pub fn assert_viewport(
  buffer: BufferArc,
  actual: &Viewport,
  expect: &Vec<&str>,
  expect_start_line: usize,
  expect_end_line: usize,
  expect_start_fills: &BTreeMap<usize, usize>,
  expect_end_fills: &BTreeMap<usize, usize>,
) {
  info!(
    "actual start_line/end_line:{:?}/{:?}",
    actual.start_line_idx(),
    actual.end_line_idx()
  );
  info!(
    "expect start_line/end_line:{:?}/{:?}",
    expect_start_line, expect_end_line
  );
  for (k, v) in actual.lines().iter() {
    info!("actual line[{:?}]: {:?}", k, v);
  }
  for (i, e) in expect.iter().enumerate() {
    info!("expect line[{}]:{:?}", i, e);
  }
  assert_eq!(expect_start_fills.len(), expect_end_fills.len());
  for (k, start_v) in expect_start_fills.iter() {
    let end_v = expect_end_fills.get(k).unwrap();
    info!(
      "expect start_fills/end_fills line[{}]:{:?}/{:?}",
      k, start_v, end_v
    );
  }

  assert_eq!(actual.start_line_idx(), expect_start_line);
  assert_eq!(actual.end_line_idx(), expect_end_line);
  if actual.lines().is_empty() {
    assert!(actual.end_line_idx() <= actual.start_line_idx());
  } else {
    let (first_line_idx, _first_line_viewport) = actual.lines().first().unwrap();
    let (last_line_idx, _last_line_viewport) = actual.lines().last().unwrap();
    assert_eq!(*first_line_idx, actual.start_line_idx());
    assert_eq!(*last_line_idx, actual.end_line_idx() - 1);
  }
  assert_eq!(
    actual.end_line_idx() - actual.start_line_idx(),
    actual.lines().len()
  );

  let buffer = lock!(buffer);
  let buflines = buffer.text().rope().lines_at(actual.start_line_idx());
  let total_lines = expect_end_line - expect_start_line;

  for (l, line) in buflines.enumerate() {
    if l >= total_lines {
      break;
    }
    let actual_line_idx = l + expect_start_line;
    let line_viewport = actual.lines().get(&actual_line_idx).unwrap();

    info!(
      "l-{:?}, actual_line_idx:{}, line_viewport:{:?}",
      actual.start_line_idx() + l,
      actual_line_idx,
      line_viewport
    );
    info!(
      "l-{:?},start_filled_cols (expect/actual):{:?}/{}, end_filled_cols (expect/actual):{:?}/{}",
      actual.start_line_idx() + l,
      expect_start_fills.get(&actual_line_idx),
      line_viewport.start_filled_cols(),
      expect_end_fills.get(&actual_line_idx),
      line_viewport.end_filled_cols()
    );
    assert_eq!(
      line_viewport.start_filled_cols(),
      *expect_start_fills.get(&actual_line_idx).unwrap()
    );
    assert_eq!(
      line_viewport.end_filled_cols(),
      *expect_end_fills.get(&actual_line_idx).unwrap()
    );

    let rows = &line_viewport.rows();
    for (r, row) in rows.iter() {
      info!("row-index-{:?}, row:{:?}", r, row);

      if r > rows.first().unwrap().0 {
        let prev_r = r - 1;
        let prev_row = rows.get(&prev_r).unwrap();
        info!(
          "row-{:?}, current[{}]:{:?}, previous[{}]:{:?}",
          r, r, row, prev_r, prev_row
        );
      }
      if r < rows.last().unwrap().0 {
        let next_r = r + 1;
        let next_row = rows.get(&next_r).unwrap();
        info!(
          "row-{:?}, current[{}]:{:?}, next[{}]:{:?}",
          r, r, row, next_r, next_row
        );
      }

      let mut payload = String::new();
      for c_idx in row.start_char_idx()..row.end_char_idx() {
        let c = line.get_char(c_idx).unwrap();
        payload.push(c);
      }
      info!(
        "row-{:?}, payload actual:{:?}, expect:{:?}",
        r, payload, expect[*r as usize]
      );
      assert_eq!(payload, expect[*r as usize]);
    }
  }
}

pub fn make_canvas(
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

pub fn assert_canvas(actual: &Canvas, expect: &[&str]) {
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

pub fn update_viewport(
  buf: BufferArc,
  window: &mut Window,
  start_line: usize,
  start_column: usize,
) -> ViewportArc {
  let buf = lock!(buf);
  let viewport = Viewport::view(
    window.options(),
    buf.text(),
    window.actual_shape(),
    start_line,
    start_column,
  );
  window.set_viewport(Viewport::to_arc(viewport));
  window.viewport()
}

fn search_viewport(
  direction: ViewportSearchDirection,
  window: Rc<RefCell<Window>>,
  buf: BufferArc,
  target_cursor_line: usize,
  target_cursor_char: usize,
  expect_start_line: usize,
  expect_start_column: usize,
) -> ViewportArc {
  let mut window = window.borrow_mut();
  let old = window.viewport();
  let buf = lock!(buf);
  let opts = *window.options();
  let (start_line, start_column) = old.search_anchor(
    direction,
    &opts,
    buf.text(),
    window.actual_shape(),
    target_cursor_line,
    target_cursor_char,
  );
  assert_eq!(start_line, expect_start_line);
  assert_eq!(start_column, expect_start_column);

  let viewport = Viewport::view(
    &opts,
    buf.text(),
    window.actual_shape(),
    start_line,
    start_column,
  );
  window.set_cursor_viewport(CursorViewport::to_arc(CursorViewport::from_position(
    &viewport,
    buf.text(),
    target_cursor_line,
    target_cursor_char,
  )));
  window.set_viewport(Viewport::to_arc(viewport));
  window.viewport()
}

pub fn search_down_viewport(
  window: Rc<RefCell<Window>>,
  buf: BufferArc,
  target_cursor_line: usize,
  target_cursor_char: usize,
  expect_start_line: usize,
  expect_start_column: usize,
) -> ViewportArc {
  search_viewport(
    ViewportSearchDirection::Down,
    window,
    buf,
    target_cursor_line,
    target_cursor_char,
    expect_start_line,
    expect_start_column,
  )
}

pub fn search_up_viewport(
  window: Rc<RefCell<Window>>,
  buf: BufferArc,
  target_cursor_line: usize,
  target_cursor_char: usize,
  expect_start_line: usize,
  expect_start_column: usize,
) -> ViewportArc {
  search_viewport(
    ViewportSearchDirection::Up,
    window,
    buf,
    target_cursor_line,
    target_cursor_char,
    expect_start_line,
    expect_start_column,
  )
}

pub fn search_left_viewport(
  window: Rc<RefCell<Window>>,
  buf: BufferArc,
  target_cursor_line: usize,
  target_cursor_char: usize,
  expect_start_line: usize,
  expect_start_column: usize,
) -> ViewportArc {
  search_viewport(
    ViewportSearchDirection::Left,
    window,
    buf,
    target_cursor_line,
    target_cursor_char,
    expect_start_line,
    expect_start_column,
  )
}

pub fn search_right_viewport(
  window: Rc<RefCell<Window>>,
  buf: BufferArc,
  target_cursor_line: usize,
  target_cursor_char: usize,
  expect_start_line: usize,
  expect_start_column: usize,
) -> ViewportArc {
  search_viewport(
    ViewportSearchDirection::Right,
    window,
    buf,
    target_cursor_line,
    target_cursor_char,
    expect_start_line,
    expect_start_column,
  )
}

mod tests_view_nowrap {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(27, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "But still it contains sever",
      "  1. When the line is small",
      "  2. When the line is too l",
      "     * The extra parts are ",
      "     * The extra parts are ",
      "",
    ];
    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!\n",
      "This is a quite simple and smal",
      "But still it contains several t",
      "  1. When the line is small eno",
      "  2. When the line is too long ",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      5,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new4() {
    test_log_init();

    let terminal_size = U16Size::new(20, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_empty_buffer(terminal_size, buf_opts);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new5() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello,\tRSVIM!\n",
        "This\r",
        "is a quite\tsimple and small test lines.\n",
        "But still\\it\r",
        "contains\tseveral things we want to test:\n",
        "\t1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "\t2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
        "\t\t* The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "\t\t* The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello,", // 4 fills for '\t'
      "This\r",
      "is a quite",
      "But still\\",
      "contains", // 2 fills for '\t'
      "\t1.",
      "\t2.",
      "\t", // 2 fills for '\t'
      "\t", // 2 fills for '\t'
      "",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
      (8, 0),
      (9, 0),
    ]
    .into_iter()
    .collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![
      (0, 4),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 2),
      (5, 0),
      (6, 0),
      (7, 2),
      (8, 2),
      (9, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      10,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new6() {
    test_log_init();

    let terminal_size = U16Size::new(27, 6);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "你好，\tRSVIM！\n",
        "这是\ta quite 简单而且很小的测试文字内容行。\n",
        "But still\\it\t包含了好几种我们想测试的情况：\n",
        "\t1. 当那条线\tis small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line 特别长而无法完全 to put in a row of the window content widget, there're multiple cases:\n",
        "\t* The extra\tparts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "  * The extra parts\tare split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "你好，\tRSVIM！\n",
      "这是\ta quite 简单而",  // 1 fills for '且'
      "But still\\it\t包含了", // 1 fills for '好'
      "\t1. 当那条线\t",
      "  2. When the line 特别长而",
      "\t* The extra\t",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 1), (2, 1), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      6,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new7() {
    test_log_init();

    let terminal_size = U16Size::new(20, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(terminal_size, buf_opts, vec![]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new8() {
    test_log_init();

    let terminal_size = U16Size::new(20, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(terminal_size, buf_opts, vec![""]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new9() {
    test_log_init();

    let terminal_size = U16Size::new(20, 20);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(terminal_size, buf_opts, vec![]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      1,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn new10() {
    test_log_init();

    let terminal_size = U16Size::new(13, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!",
      "This is a qui",
      "But still it ",
      "  1. When the",
      "  2. When the",
      "     * The ex",
      "     * The ex",
      "",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }
}

mod tests_view_nowrap_wineol {
  use super::*;

  #[test]
  fn new1_unix() {
    test_log_init();

    let terminal_size = U16Size::new(33, 5);
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Unix)
      .build()
      .unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\r\n",
        "This is a quite simple lines.\r\n",
        "But still it contains several things.\r\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple lines.\r\n",
      "But still it contains several thi",
      "",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      4,
      &expect_fills,
      &expect_fills,
    );
  }

  // #[test]
  fn _new1_win() {
    test_log_init();

    let terminal_size = U16Size::new(33, 5);
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Dos)
      .build()
      .unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\r\n",
        "This is a quite simple lines.\r\n",
        "But still it contains several things:\r\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!\r\n",
      "This is a quite simple lines.\r\n",
      "But still it contains several thin",
      "",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      4,
      &expect_fills,
      &expect_fills,
    );
  }
}

mod tests_view_nowrap_startcol {
  use super::*;

  #[test]
  fn update1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "lo, RSVIM!",
      "s is a qui",
      " still it ",
      ". When the",
      ". When the",
      "  * The ex",
      "  * The ex",
      "",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 3);
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      " RSVIM!\n",
      "s a quite ",
      "ill it con",
      "hen the li",
      "hen the li",
      " The extra",
      " The extra",
      "",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 6);
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "",
      " simple an",
      "ntains sev",
      "ine is sma",
      "ine is too",
      "a parts ar",
      "a parts ar",
      "",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 15);
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "",
      "",
      "",
      "a row of t",
      " of the wi",
      "and word-w",
      "r line-wra",
      "",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 60);
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update5() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
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

    let expect = vec!["", "", "", "", "", "", "", ""];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 500);
    let expect_fills: BTreeMap<usize, usize> = vec![
      (0, 0),
      (1, 0),
      (2, 0),
      (3, 0),
      (4, 0),
      (5, 0),
      (6, 0),
      (7, 0),
    ]
    .into_iter()
    .collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      8,
      &expect_fills,
      &expect_fills,
    );
  }
}

mod tests_view_wrap_nolinebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "IM!\n",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.\n",
      "But still ",
      "it contain",
      "s several ",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn new2() {
    let terminal_size = U16Size::new(27, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains sever",
      "al things we want to test:\n",
      "  1. When the line is small",
      " enough to completely put i",
      "nside a row of the window c",
      "ontent widget, then the lin",
      "e-wrap and word-wrap doesn'",
      "t affect the rendering.\n",
      "  2. When the line is too l",
      "ong to be completely put in",
      " a row of the window conten",
      "t widget, there're multiple",
      "",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      5,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new3() {
    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!\n",
      "This is a quite simple and smal",
      "l test lines.\n",
      "But still it contains several t",
      "hings we want to test:\n",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn new4() {
    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_empty_buffer(terminal_size, buf_opts);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn new5() {
    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "\t\t* The extra parts are\tsplit into the next\trow,\tif either line-wrap or word-wrap options are been set. If the extra\tparts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "\t\t* The extra par",
      "ts are\tsplit into the ne",
      "xt\trow,\tif either",
      " line-wrap or word-wrap options",
      " are been set. If the extra",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 4)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new6() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t\t1. When\tthe line\tis small\tenough to\tcompletely put\tinside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      ],
    );
    let expect = vec![
      "But still it contains several t",
      "hings we want to test:\n",
      "\t\t1. When\t",
      "the line\tis small",
      "\tenough to\tcomple",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new7() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t\t1. When\tthe line\tis small\tenough\tto\tcompletely put\tinside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      ],
    );
    let expect = vec![
      "But still it contains several t",
      "hings we want to test:\n",
      "\t\t1. When\t",
      "the line\tis small",
      "\tenough\tto", // 7 fills
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 7)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new8() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "但它仍然contains several things 我们想要测试的文字内容：\n",
        "\t第一，当一行文字内容太小了，然后可以完全的放进窗口的一行之中，那么行wrap和词wrap两个选项并不会影响渲染的最终效果。\n",
      ],
    );
    let expect = vec![
      "但它仍然contains several things",
      " 我们想要测试的文字内容：\n",
      "\t第一，当一行文字内容太",
      "小了，然后可以完全的放进窗口的",
      "一行之中，那么行wrap和词wrap两", // 1 fills
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new9() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "但它仍然contains several th\tings 我们想要测试的文字内容：\n",
        "\t第一，当一行文字内容太小了，然后可以完全的放进窗口的一行之中，那么行wrap和词wrap两个选项并不会影响渲染的最终效果。\n",
      ],
    );
    let expect = vec![
      "但它仍然contains several th",
      "\tings 我们想要测试的文字",
      "内容：\n",
      "\t第一，当一行文字内容太",
      "小了，然后可以完全的放进窗口的",
      "一行之中，那么行wrap和词wrap两", // 1 fills
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new10() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(terminal_size, buf_opts, vec![]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new11() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_empty_buffer(terminal_size, buf_opts);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new12() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(terminal_size, buf_opts, vec![""]);
    let expect = vec![""];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new13() {
    test_log_init();

    let terminal_size = U16Size::new(13, 8);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!",
      "This is a qui",
      "te simple and",
      " small test l",
      "ines.\n",
      "But still it ",
      "contains seve",
      "ral things we",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn new14() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "AAAAAAAAAA\n",
        "1st.\n",
        "BBBBBBBBBBCCCCCCCCCC\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ],
    );
    let expect = vec![
      "AAAAAAAAAA",
      "1st.\n",
      "BBBBBBBBBB",
      "CCCCCCCCCC",
      "3rd.\n",
      "4th.\n",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    assert_viewport(buf, &actual, &expect, 0, 5, &expect_fills, &expect_fills);
  }

  #[test]
  fn new15() {
    test_log_init();

    let terminal_size = U16Size::new(10, 6);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "1st.\n",
        "BBBBBBBBBBCCCCCCCCCC\n",
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ],
    );
    let expect = vec![
      "1st.\n",
      "BBBBBBBBBB",
      "CCCCCCCCCC",
      "3rd.\n",
      "4th.\n",
      "5th.\n",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    assert_viewport(buf, &actual, &expect, 0, 5, &expect_fills, &expect_fills);
  }

  #[test]
  fn update1() {
    let terminal_size = U16Size::new(15, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!\n",
      "This is a quite",
      " simple and sma",
      "ll test lines.\n",
      "But still it co",
      "ntains several ",
      "things we want ",
      "to test:\n",
      "  1. When the l",
      "ine is small en",
      "ough to complet",
      "ely put inside ",
      "a row of the wi",
      "ndow content wi",
      "dget, then the ",
    ];
    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      4,
      &expect_fills,
      &expect_fills,
    );

    let expect = vec![
      "But still it co",
      "ntains several ",
      "things we want ",
      "to test:\n",
      "  1. When the l",
      "ine is small en",
      "ough to complet",
      "ely put inside ",
      "a row of the wi",
      "ndow content wi",
      "dget, then the ",
      "line-wrap and w",
      "ord-wrap doesn'",
      "t affect the re",
      "ndering.\n",
    ];
    let actual = update_viewport(buf.clone(), &mut window, 2, 0);
    let expect_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      2,
      4,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update2() {
    let terminal_size = U16Size::new(15, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!\n",
      "This is a quite",
      " simple and sma",
      "ll test lines.\n",
      "But still it co",
      "ntains several ",
      "things we want ",
      "to test:\n",
      "  1. When the l",
      "ine is small en",
      "ough to complet",
      "ely put inside ",
      "a row of the wi",
      "ndow content wi",
      "dget, then the ",
    ];
    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      4,
      &expect_fills,
      &expect_fills,
    );

    let expect = vec![
      "     * The extr",
      "a parts are spl",
      "it into the nex",
      "t row, if eithe",
      "r line-wrap or ",
      "word-wrap optio",
      "ns are been set",
      ". If the extra ",
      "parts are still",
      " too long to pu",
      "t in the next r",
      "ow, repeat this",
      " operation agai",
      "n and again. Th",
      "is operation al",
    ];
    let actual = update_viewport(buf.clone(), &mut window, 6, 0);
    let expect_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      6,
      7,
      &expect_fills,
      &expect_fills,
    );
  }

  #[test]
  fn update3() {
    let terminal_size = U16Size::new(15, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
      ],
    );

    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite",
      " simple and sma",
      "ll test lines.\n",
      "",
    ];
    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      0,
      3,
      &expect_fills,
      &expect_fills,
    );

    let expect = vec!["This is a quite", " simple and sma", "ll test lines.\n", ""];
    let actual = update_viewport(buf.clone(), &mut window, 1, 0);
    let expect_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buf.clone(),
      &actual,
      &expect,
      1,
      3,
      &expect_fills,
      &expect_fills,
    );
  }
}

mod tests_view_wrap_nolinebreak_wineol {
  use super::*;

  #[test]
  fn new1_unix() {
    test_log_init();

    let terminal_size = U16Size::new(10, 13);
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Unix)
      .build()
      .unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\r\n",
        "This is a quite simple and small test lines.\r\n",
        "But still it contains several.\r\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r\n",
      ],
    );
    let expect = vec![
      "Hello, RSV",
      "IM!\r\n",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.\r\n",
      "But still ",
      "it contain",
      "s several.",
      "\r\n",
      "  1. When ",
      "the line i",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 4, &expect_fills, &expect_fills);
  }

  // #[test]
  fn _new1_win() {
    test_log_init();

    let terminal_size = U16Size::new(10, 13);
    let buf_opts = BufferLocalOptionsBuilder::default()
      .file_format(FileFormatOption::Dos)
      .build()
      .unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\r\n",
        "This is a quite simple and small test lines.\r\n",
        "But still it contains several.\r\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\r\n",
        "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\r\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\r\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\r\n",
      ],
    );
    let expect = vec![
      "Hello, RSV",
      "IM!\r\n",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.\r\n",
      "But still ",
      "it contain",
      "s several.",
      "  1. When ",
      "the line i",
      "s too long",
    ];

    let window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 4, &expect_fills, &expect_fills);
  }
}

mod tests_view_wrap_nolinebreak_startcol {
  use super::*;

  #[test]
  fn update1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "lo, RSVIM!",
      "s is a qui",
      "te simple ",
      "and small ",
      "test lines",
      ".\n",
      " still it ",
      "contains s",
      "everal thi",
      "ngs we wan",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 3);
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn update2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "lo, RSVIM!",
      "s is a qui",
      "te simple ",
      "and small ",
      "test lines",
      ".\n",
      " still it ",
      "contains s",
      "everal thi",
      "ngs we wan",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 3);
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 3, &expect_fills, &expect_fills);
  }

  #[test]
  fn update3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "",
      " simple an",
      "d small te",
      "st lines.\n",
      "ntains sev",
      "eral thing",
      "s we want ",
      "to test:\n",
      "ine is sma",
      "ll enough ",
      "to complet",
      "ely put in",
      "side a row",
      " of the wi",
      "ndow conte",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 15);
    let expect_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 0, 4, &expect_fills, &expect_fills);
  }

  #[test]
  fn update4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "",
      "",
      "a row of t",
      "he window ",
      "content wi",
      "dget, then",
      " the line-",
      "wrap and w",
      "ord-wrap d",
      "oesn't aff",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 1, 60);
    let expect_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(buf, &actual, &expect, 1, 4, &expect_fills, &expect_fills);
  }

  #[test]
  fn update5() {
    test_log_init();

    let terminal_size = U16Size::new(31, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "但它仍然contains several th\tings 我们想要测试的文字内容：\n",
        "\t第一，当一行文字内容太小了，然后可以完全的放进窗口的一行之中，那么行wrap和词wrap两个选项并不会影响渲染的最终效果。\n",
      ],
    );
    let expect = vec![
      "ins several th\tings 我们",
      "想要测试的文字内容：\n",
      "当一行文字内容太小了，然后可以",
      "完全的放进窗口的一行之中，那么",
      "行wrap和词wrap两个选项并不会影",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 13);
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 1)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      2,
      &expect_start_fills,
      &expect_end_fills,
    );
  }
}

mod tests_view_wrap_linebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
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
      "Hello, ",
      "RSVIM!\n",
      "This is a ",
      "quite ",
      "simple and",
      " small ",
      "test lines",
      ".\n",
      "But still ",
      "it ",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      3,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(27, 15);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
        "  2. When the line is to\to long to be completely p\tut in a row of the window content widget, there're multiple cases:\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains ",
      "several things we want to ",
      "test:\n",
      "  1. When the line is small",
      " enough to completely put ",
      "inside a row of the window ",
      "content widget, then the ",
      "line-wrap and word-wrap ",
      "doesn't affect the ",
      "rendering.\n",
      "  2. When the line is to",
      "\to long to be ",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
      .into_iter()
      .collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      5,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(31, 11);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "  1. When the line is small ",
      "enough to completely put inside",
      " a row of the window content ",
      "widget, then the line-wrap and ",
      "word-wrap doesn't affect the ",
      "rendering.\n",
      "",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_empty_buffer(terminal_size, buf_opts);
    let expect = vec![""];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(buffer, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn new5() {
    let terminal_size = U16Size::new(31, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "  1. When the line is small ",
      "enough to completely put inside",
      " a row of the window content ",
      "widget, then the line-wrap and ",
      "word-wrap doesn't affect the ",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new6() {
    test_log_init();

    let terminal_size = U16Size::new(31, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
        "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "\t\t第一，当一行文",
      "本内容的长度足够短，短到可以完",
      "整的放入一个窗口（的一行）之中",
      "，那么基于行的换行和基于单词的",
      "换行两个选项都不会影响渲染的最",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new7() {
    test_log_init();

    let terminal_size = U16Size::new(31, 11);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
        "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and ",
      "small test lines.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "\t\t第一，当一行文",
      "本内容的长度足够短，短到可以完",
      "整的放入一个窗口（的一行）之中",
      "，那么基于行的换行和基于单词的",
      "换行两个选项都不会影响渲染的最",
      "终效果。\n",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new8() {
    test_log_init();

    let terminal_size = U16Size::new(31, 11);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
        "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "Hello, RSVIM!\n",
      "This is a quite simple ",
      "andsmalltestlineswithoutevenane",
      "wlinebreakbecausewewanttotestho",
      "witwillhappensifthereisaverylon",
      "gwordthatcannotbeenpplaceinside",
      "arowofthewindowcontent.\n",
      "But still it contains several ",
      "things we want to test:\n",
      "\t\t第一，当一行文",
      "本内容的长度足够短，短到可以完",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new9() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
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
      "Hello, ",
      "RSVIM!\n",
      "This is a ",
      "quite ",
      "simple and",
      " small ",
      "test lines",
      ".\n",
      "But still ",
      "it ",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      3,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new10() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
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
      "Hello, ",
      "RSVIM!\n",
      "This is a ",
      "quite ",
      "simple and",
      " small ",
      "test lines",
      ".\n",
      "But still ",
      "it contai",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      3,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new11() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
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
      "This is a ",
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
      "ent.\n",
      "But still it ",
      "contains ",
      "several ",
      "things we ",
      "want to test:",
      "  1. When the",
      " line is ",
      "small enough ",
      "to completely",
      " put inside a",
      " row of the ",
      "window ",
      "content ",
      "widget, 那么",
      "行换行和单词",
      "换行选项都不",
      "会影响最终的",
      "渲染效果。\n",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new12() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
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
      "This is a ",
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
      "ent.\n",
      "But still it ",
      "contains ",
      "several ",
      "things we ",
      "want to test:",
      "  1. When the",
      " line is ",
      "small enough ",
      "to completely",
      " put inside a",
      " row of the ",
      "window ",
      "content ",
      "widget, 那么",
      "行换行和单词",
      "换行选项都不",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn new13() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(terminal_size, buf_opts, vec![]);
    let expect = vec![""];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(buffer, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn new14() {
    test_log_init();

    let terminal_size = U16Size::new(13, 31);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(terminal_size, buf_opts, vec![""]);
    let expect = vec![""];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(buffer, &actual, &expect, 0, 1, &expect_fills, &expect_fills);
  }

  #[test]
  fn new15() {
    test_log_init();

    let terminal_size = U16Size::new(13, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buffer = make_buffer_from_lines(
      terminal_size,
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
      "Hello, RSVIM!",
      "This is a ",
      "quite simple ",
      "and small ",
      "test lines.\n",
      "But still it ",
      "contains ",
      "several ",
      "things we ",
      "want to test:",
    ];

    let window = make_window(terminal_size, buffer.clone(), &win_opts);
    let actual = window.viewport();
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buffer,
      &actual,
      &expect,
      0,
      3,
      &expect_start_fills,
      &expect_end_fills,
    );
  }
}

mod tests_view_wrap_linebreak_startcol {
  use super::*;

  #[test]
  fn update1() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "lo, RSVIM!",
      "s is a ",
      "quite ",
      "simple and",
      " small ",
      "test lines",
      ".\n",
      " still it ",
      "contains ",
      "several ",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 3);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      3,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn update2() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      " RSVIM!\n",
      "s a quite ",
      "simple and",
      " small ",
      "test lines",
      ".\n",
      "ill it ",
      "contains ",
      "several ",
      "things we ",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 6);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      3,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn update3() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "",
      "le and ",
      "small test",
      " lines.\n",
      "s several ",
      "things we ",
      "want to ",
      "test:\n",
      "s small ",
      "enough to ",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 20);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn update4() {
    test_log_init();

    let terminal_size = U16Size::new(10, 10);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
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
      "",
      "",
      "",
      "a row of ",
      "the window",
      " content ",
      "widget, ",
      "then the ",
      "line-wrap ",
      "and word-",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 60);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn update5() {
    test_log_init();

    let terminal_size = U16Size::new(31, 11);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple andsmalltestlineswithoutevenanewlinebreakbecausewewanttotesthowitwillhappensifthereisaverylongwordthatcannotbeenpplaceinsidearowofthewindowcontent.\n",
        "But still it contains several things we want to test:\n",
        "\t\t第一，当一行文本内容的长度足够短，短到可以完整的放入一个窗口（的一行）之中，那么基于行的换行和基于单词的换行两个选项都不会影响渲染的最终效果。\n",
        "\t\t第二，当一行内容文本的长度足够长，而无法放入窗口中，那么我们需要考虑很多种情况：\n",
        "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
        "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );
    let expect = vec![
      "",
      " simple ",
      "andsmalltestlineswithoutevenane",
      "wlinebreakbecausewewanttotestho",
      "witwillhappensifthereisaverylon",
      "gwordthatcannotbeenpplaceinside",
      "arowofthewindowcontent.\n",
      "ntains several things we want ",
      "to test:\n",
      "第一，当一行文本内容的长度足够",
      "短，短到可以完整的放入一个窗口",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 15);
    let expect_start_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 1)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> =
      vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      4,
      &expect_start_fills,
      &expect_end_fills,
    );
  }

  #[test]
  fn update6() {
    test_log_init();

    let terminal_size = U16Size::new(17, 4);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "1. When the line contains some super long long word that cannot put, wewillhavetofallbacktonolinebreakbehaviorandthustrytogetmoresmoothbehavior thus to make a more smooth and eye friendly moving or scrolling behavior.\n",
      ],
    );
    let expect = vec![
      "ewillhavetofallba",
      "cktonolinebreakbe",
      "haviorandthustryt",
      "ogetmoresmoothbeh",
    ];

    let mut window = make_window(terminal_size, buf.clone(), &win_opts);
    let actual = update_viewport(buf.clone(), &mut window, 0, 70);
    let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
    assert_viewport(
      buf,
      &actual,
      &expect,
      0,
      1,
      &expect_start_fills,
      &expect_end_fills,
    );
  }
}

mod tests_search_anchor_downward_nowrap {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];
      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 15, 0, 0);
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 1, 1, 0);
      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 2), (4, 2), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["ut still it conta", "1. When", "2. When", "\t3.", "\t4."];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 3, 2, 1);
      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 7), (4, 7), (5, 7), (6, 7)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 3), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 3, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "",
        "nd small test lin",
        "veral things we w",
        "he\tline",
        "t\t\t",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 40, 0, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "", "t\tinside.\n", ""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 130, 0, 113);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", "", "", "mpletely\tp", ":\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 100, 0, 95);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["", "", "", "", "not\tset.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 100, 1, 146);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 1)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec!["", "\tcompletel", "put:\n", "\tand", "if\teither"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 50, 2, 85);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 7), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 6), (6, 1)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "",
        "nd small test lin",
        "veral things we w",
        "he\tline",
        "t\t\t",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 40, 0, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "", "t\tinside.\n", ""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 130, 0, 113);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", "", "", "to\tcom", "etely\tput:"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 30, 0, 79);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 4), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["", "", "inside.\n", "", "options\ta"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 80, 1, 120);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 2), (4, 0), (5, 1)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "l it contains sev",
        "1. When\tth",
        "2. When\tit",
        "\t3. The ex",
        "\t4. The ex",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 1, 2, 8);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new4() {
    test_log_init();

    let terminal_size = U16Size::new(17, 4);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "1. When the line is small enough to completely put inside.\n",
        "2. When it too long to completely put:\n",
        "\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. What\tif\tthe\textra\tparts\tare\tstill\ttoo\tlong?\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "1. When the line ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "1. When the line ",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 0, 12, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "",
        "small test lines.",
        "al things we want",
        "nough to complete",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 43, 0, 27);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "",
        "mall test lines.\n",
        "l things we want ",
        "ough to completel",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 44, 0, 28);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["", "t lines.\n", " we want to test:", "completely put in"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 52, 0, 36);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec!["", " lines.\n", "we want to test:\n", "ompletely put ins"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 53, 0, 37);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["", "s.\n", "nt to test:\n", "tely put inside.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 60, 0, 42);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        " lines.\n",
        "we want to test:\n",
        "ompletely put ins",
        ":\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 40, 1, 37);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new5() {
    test_log_init();

    let terminal_size = U16Size::new(17, 4);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "1. When the line is small enough to completely put inside.\n",
        "2. When it too long to completely put:\n",
        "\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. What\tif\tthe\textra\tparts\tare\tstill\ttoo\tlong?\n",
        "5. This is the last line.",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "1. When the line ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        " and small test l",
        "several things we",
        "all enough to com",
        " completely put:\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 40, 1, 22);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "", "not\tset.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 99, 2, 138);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 1)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", "", "", "too\tlong?\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 168, 3, 271);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0), (5, 0), (6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0), (5, 0), (6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "ompletely put:\n",
        "ts are been trunc",
        "ts are split into",
        ".",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 25, 4, 24);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}

mod tests_search_anchor_downward_wrap_nolinebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 15, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "small\t",
        "enough\tto",
        "\tcompletel",
        "y\tput",
        "\tinside.\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 60, 3, 52);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 35, 4, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\t",
        "line-wrap\t",
        "and\tword-w",
        "rap\toption",
        "s\tare",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 82, 5, 59);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "if\t",
        "either\tlin",
        "e-wrap\tor",
        "\tword-wrap",
        "\toptions",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 82, 6, 78);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 43, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "small\t",
        "enough\tto",
        "\tcompletel",
        "y\tput",
        "\tinside.\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 58, 3, 52);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 30, 4, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\t",
        "line-wrap\t",
        "and\tword-w",
        "rap\toption",
        "s\tare",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 82, 5, 59);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "xtra parts are sp",
        "lit into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 10, 6, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["line\t", "is\tsmall", "\tenough", "\tto", "\tcompletel"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 37, 1, 29);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 37, 2, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new4() {
    test_log_init();

    let terminal_size = U16Size::new(17, 4);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if both line-wrap and word-wrap options\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
        "5. The extra parts are split into the next row if either options are set.\n",
        "6. The extra parts are split into the next row if either line-wrap or word-wrap options are set.\n",
        "7. The extra parts...",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 0, 53, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["enough\t", "to\tcomplet", "ely\tput", "\tinside.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 58, 1, 66);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 3)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\t", "long\tto", "\tcompletel", "y\tput:\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 58, 2, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["wrap and word-wra", "p options\t", "are\tnot", "\tset.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 97, 3, 67);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        " rows in the wind",
        "ow, thus it may c",
        "ontains less line",
        "s in the buffer.\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 314, 4, 305);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        " extra parts are ",
        "split into the ne",
        "xt row if either ",
        "options are set.\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 314, 5, 6);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "into the next row",
        " if either line-w",
        "rap or word-wrap ",
        "options are set.\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 314, 6, 29);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec!["7. The extra part", "s...", "", ""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 314, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new5() {
    test_log_init();

    let terminal_size = U16Size::new(10, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "1st.\n",
        "AAAAAAAAAABBBBBBBBBB\n", // exactly 2 rows
        "3rd.\n",
        "4th.\n",
        "5th.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec!["1st.\n", "AAAAAAAAAA", "BBBBBBBBBB", "3rd.\n", "4th.\n"];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["1st.\n", "AAAAAAAAAA", "BBBBBBBBBB", "3rd.\n", "4th.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 20, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new6() {
    test_log_init();

    let terminal_size = U16Size::new(10, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "1st.\n",
        "2nd.\n",
        "3rd.\n",
        "AAAAAAAAAABBBBBBBBBB\n", // exactly 2 rows
        "5th.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec!["1st.\n", "2nd.\n", "3rd.\n", "AAAAAAAAAA", "BBBBBBBBBB"];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["2nd.\n", "3rd.\n", "AAAAAAAAAA", "BBBBBBBBBB", "5th.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 20, 1, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}

mod tests_search_anchor_downward_wrap_linebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 15, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["enough\t", "to\t", "completely", "\tput", "\tinside.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 60, 3, 66);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\t", "long\tto", "\t", "completely", "\tput:\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 35, 4, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline",
        "-wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 82, 5, 63);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 1)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec!["if\t", "either\t", "line-wrap\t", "or\tword-", "wrap\t"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 82, 6, 78);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 43, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["enough\t", "to\t", "completely", "\tput", "\tinside.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 58, 3, 66);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\t", "long\tto", "\t", "completely", "\tput:\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 30, 4, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "both\tline",
        "-wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 82, 5, 63);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 1)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "xtra parts are ",
        "split into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 10, 6, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["is\t", "small\t", "enough\tto", "\t", "completely"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 37, 1, 39);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 7)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["too\t", "long\tto", "\t", "completely", "\tput:\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 37, 2, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "are been ",
        "truncated if",
        "\tboth",
        "\tline-wrap",
        "\tand",
        // "\tword-wrap",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 30, 3, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["are split into ", "the\tnext", "\trow,", "\tif", "\teither"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 30, 4, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 5, 82, 5, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new4() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["enough\t", "to\t", "completely", "\tput", "\tinside.\n"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 70, 1, 66);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 3)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}

mod tests_search_anchor_upward_nowrap {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["is\tsmall", "long", "runcated if", "into the\tn", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 6, 40, 3, 45);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 1), (4, 7), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 1), (4, 6), (5, 6), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["to\tcom", "etely\tput:", "e-wrap\tand", "if\te", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 5, 60, 3, 79);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 4), (4, 0), (5, 0), (6, 6), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["to\tcom", "etely\tput:", "e-wrap\tand", "if\te", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 4, 38, 3, 79);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 4), (4, 0), (5, 0), (6, 6), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["put\tinsi", "", "wrap\toptio", "line-wrap\t", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 3, 55, 3, 109);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "things we want to",
        "line\ti",
        "\ttoo",
        "arts are been tru",
        "arts are split in",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 2, 30, 2, 30);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 4), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "ll test lines.\n",
        "things we want to",
        "line\ti",
        "\ttoo",
        "arts are been tru",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 1, 32, 1, 30);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 4), (4, 3), (5, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 3), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "SVIM!\n",
        "a quite simple an",
        "l it contains sev",
        "1. When\tth",
        "2. When\tit",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 0, 8, 0, 8);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec!["\t1. When", "\t2. When", "\t\t3", "\t\t4", ""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 2), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["put\tinsid", "", "rap\toption", "ine-wrap\to", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 6, 70, 3, 110);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 1), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["inside.\n", "", "options\ta", "or\tw", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 5, 80, 3, 120);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 2), (4, 0), (5, 1), (6, 6), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["o\tcomplete", "\tput:\n", "p\tand", "if\teither", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 4, 35, 3, 84);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 1), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 5), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["o\tcomplete", "\tput:\n", "p\tand", "if\teither", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 3, 36, 3, 84);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 0), (6, 1), (7, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0), (4, 0), (5, 5), (6, 0), (7, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "things we want to",
        "line\ti",
        "\ttoo",
        "arts are been tru",
        "arts are split in",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 2, 30, 2, 30);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 4), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "ll test lines.\n",
        "things we want to",
        "line\ti",
        "\ttoo",
        "arts are been tru",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 1, 32, 1, 30);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 4), (4, 3), (5, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0), (3, 0), (4, 3), (5, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "SVIM!\n",
        "a quite simple an",
        "l it contains sev",
        "1. When\tth",
        "2. When\tit",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 0, 8, 0, 8);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 4);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
        "5. This is the last line.",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "2. When\tit",
        "\t3. The ex",
        "\t4. The ex",
        "is the last line.",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 24, 4, 8);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1.1
    {
      let expect = vec![
        "2. When\ti",
        "\t3. The e",
        "\t4. The e",
        " is the last line",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 7, 7, 4, 7);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 1), (5, 1), (6, 1), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "s in the buffer.\n", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 6, 318, 4, 377);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2.1
    {
      let expect = vec!["", "", "es in the buffer.", ""];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 285, 4, 376);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", ".\n", "are", ""];

      let actual = search_left_viewport(window.clone(), buf.clone(), 5, 102, 4, 161);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 7), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 7), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3.1
    {
      let expect = vec!["", "t.\n", "\tare", ""];

      let actual = search_left_viewport(window.clone(), buf.clone(), 5, 90, 4, 160);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 6), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![":\n", "d\tword-wra", "either\tlin", ""];

      let actual = search_up_viewport(window.clone(), buf.clone(), 4, 48, 4, 95);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec!["mpletely\tp", ":\n", "d\tword-wra", "either\tlin"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 3, 48, 3, 95);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0), (5, 0), (6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0), (5, 0), (6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["test:\n", "\tsmall", "long\tt", "cated if\tb"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 2, 48, 2, 48);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 0), (4, 4), (5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(2, 0), (3, 4), (4, 0), (5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![".\n", "t to test:\n", "is\tsmal", "o\tlong"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 1, 48, 1, 43);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 3), (4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(1, 0), (2, 0), (3, 0), (4, 4)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec!["!\n", "ite simple and sm", " contains several", "hen\tthe"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 0, 48, 0, 12);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}

mod tests_search_anchor_upward_wrap_nolinebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "nd again. This op",
        "eration also eats",
        " more rows in the",
        " window, thus it ",
        "may contains less",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 6, 280, 6, 287);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["and\t", "word-wrap\t", "options\tar", "e\tnot", "\tset.\n"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 5, 60, 5, 87);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 4, 35, 4, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "small\t",
        "enough\tto",
        "\tcompletel",
        "y\tput",
        "\tinside.\n",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 3, 82, 3, 52);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 2, 82, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
        "ains several thin",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 1, 0, 1, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 0, 8, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["next\t", "row,\tif", "\teither", "\tline-wrap", "\tor"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 6, 70, 6, 56);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "both\t",
        "line-wrap\t",
        "and\tword-w",
        "rap\toption",
        "s\tare",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 5, 80, 5, 59);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["t\t\t", "too\tlong", "\tto", "\tcompletel", "y\tput:\n"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 4, 35, 4, 24);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["line\t", "is\tsmall", "\tenough", "\tto", "\tcompletel"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 3, 36, 3, 29);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "But still it cont",
        "ains several thin",
        "gs we want to tes",
        "t:\n",
        "\t1. When",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 2, 30, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
        "ains several thin",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 1, 32, 1, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 0, 8, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(21, 7);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a simple test.\n",
        "But still it contains several cases:\n",
        " 1. When the line is small.\n",
        " 2. When it is too long:\n",
        "  2.1. The extra parts are truncated.\n",
        "  2.2. The extra parts are splitted.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a simple test",
        ".\n",
        "But still it contains",
        " several cases:\n",
        " 1. When the line is ",
        "small.\n",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a simple test",
        ".\n",
        "But still it contains",
        " several cases:\n",
        " 1. When the line is ",
        "small.\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 0, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a simple test",
        ".\n",
        "But still it contains",
        " several cases:\n",
        " 1. When the line is ",
        "small.\n",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 2, 0, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a simple test",
        ".\n",
        "But still it contains",
        " several cases:\n",
        " 1. When the line is ",
        "small.\n",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 1, 0, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a simple test",
        ".\n",
        "But still it contains",
        " several cases:\n",
        " 1. When the line is ",
        "small.\n",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 0, 0, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        " 2. When it is too lo",
        "ng:\n",
        "  2.1. The extra part",
        "s are truncated.\n",
        "  2.2. The extra part",
        "s are splitted.\n",
        "",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 0, 4, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        " 2. When it is too lo",
        "ng:\n",
        "  2.1. The extra part",
        "s are truncated.\n",
        "  2.2. The extra part",
        "s are splitted.\n",
        "",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 5, 0, 4, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        " 2. When it is too lo",
        "ng:\n",
        "  2.1. The extra part",
        "s are truncated.\n",
        "  2.2. The extra part",
        "s are splitted.\n",
        "",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 4, 0, 4, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(4, 0), (5, 0), (6, 0), (7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec![
        " 1. When the line is ",
        "small.\n",
        " 2. When it is too lo",
        "ng:\n",
        "  2.1. The extra part",
        "s are truncated.\n",
        "  2.2. The extra part",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 3, 0, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0), (5, 0), (6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(3, 0), (4, 0), (5, 0), (6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}

mod tests_search_anchor_upward_wrap_linebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "eats more rows in",
        " the window, thus",
        " it may contains ",
        "less lines in the",
        " buffer.\n",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 6, 295, 6, 317);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["and\t", "word-wrap\t", "options\t", "are\tnot", "\tset.\n"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 5, 60, 5, 87);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      info!("actual:{:?}", actual);
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\t", "long\tto", "\t", "completely", "\tput:\n"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 4, 35, 4, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["enough\t", "to\t", "completely", "\tput", "\tinside.\n"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 3, 82, 3, 66);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 2, 82, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
        "contains several ",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 1, 0, 1, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 0, 8, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![""];

      let actual = search_down_viewport(window.clone(), buf.clone(), 7, 0, 7, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(7, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        7,
        8,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["next\t", "row,\tif", "\teither", "\tline-wrap", "\tor"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 6, 70, 6, 56);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "both\tline",
        "-wrap\tand",
        "\tword-wrap",
        "\toptions",
        "\tare",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 5, 80, 5, 63);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 1)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["too\t", "long\tto", "\t", "completely", "\tput:\n"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 4, 35, 4, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["ine\tis", "\tsmall", "\tenough", "\tto", "\t"];

      let actual = search_up_viewport(window.clone(), buf.clone(), 3, 36, 3, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "But still it ",
        "contains several ",
        "things we want to",
        " test:\n",
        "\t1. When",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 2, 30, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
        "contains several ",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 1, 32, 1, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 0, 8, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(21, 6);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
        "5. When the line is small enough to completely put inside.\n",
        "6. When it too long to completely put:\n",
        "7. The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small test",
        " lines.\n",
        "But still it contains",
        " several things we ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );

      let expect_canvas = vec![
        "Hello, RSVIM!        ",
        "This is a quite      ",
        "simple and small test",
        " lines.              ",
        "But still it contains",
        " several things we   ",
      ];

      let actual_canvas = make_canvas(
        terminal_size,
        win_opts,
        buf.clone(),
        window.borrow().viewport(),
      );
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Prepare
    {
      let expect = vec![
        "7. The extra parts ",
        "are been truncated if",
        " both line-wrap and ",
        "word-wrap options are",
        " not set.\n",
        "",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 10, 0, 9, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(9, 0), (10, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(9, 0), (10, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        9,
        11,
        &expect_start_fills,
        &expect_end_fills,
      );

      let expect_canvas = vec![
        "7. The extra parts   ",
        "are been truncated if",
        " both line-wrap and  ",
        "word-wrap options are",
        " not set.            ",
        "                     ",
      ];

      let actual_canvas = make_canvas(
        terminal_size,
        win_opts,
        buf.clone(),
        window.borrow().viewport(),
      );
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Search-1
    {
      let expect = vec![
        "extra parts are split",
        " into the\tnext",
        "\trow,\t",
        "if\teither",
        "\tline-wrap",
        "\tor\t",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 6, 70, 6, 23);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );

      let expect_canvas = vec![
        "extra parts are split",
        " into the        next",
        "        row,         ",
        "if        either     ",
        "        line-wrap    ",
        "        or           ",
      ];

      let actual_canvas = make_canvas(
        terminal_size,
        win_opts,
        buf.clone(),
        window.borrow().viewport(),
      );
      assert_canvas(&actual_canvas, &expect_canvas);
    }

    // Search-2
    {
      let expect = vec![
        "extra parts are been ",
        "truncated if\t",
        "both\tline-wrap",
        "\tand\t",
        "word-wrap\t",
        "options\tare",
      ];

      let actual = search_up_viewport(window.clone(), buf.clone(), 5, 80, 5, 23);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(5, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        5,
        6,
        &expect_start_fills,
        &expect_end_fills,
      );

      let expect_canvas = vec![
        "extra parts are been ",
        "truncated if         ",
        "both        line-wrap",
        "        and          ",
        "word-wrap            ",
        "options        are   ",
      ];

      let actual_canvas = make_canvas(
        terminal_size,
        win_opts,
        buf.clone(),
        window.borrow().viewport(),
      );
      assert_canvas(&actual_canvas, &expect_canvas);
    }
  }
}

mod tests_search_anchor_horizontally_nowrap {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
        "\t\t4",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 0, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 2), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
        "\t\t4",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 1, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 2), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["ut still it conta", "1. When", "2. When", "\t3.", "\t4."];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 3, 2, 1);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 7), (4, 7), (5, 7), (6, 7)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 3), (4, 3), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        " we want to test:",
        "ne\tis",
        "too\tl",
        "re been truncated",
        "re split into the",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 38, 2, 36);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 5), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 5), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        " to test:\n",
        "is\tsmall",
        "\tlong",
        "truncated if",
        " into the\t",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 39, 2, 44);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 5), (5, 5), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "to test:\n",
        "is\tsmall",
        "long",
        "runcated if",
        "into the\tn",
        "\t\t3",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 40, 2, 45);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 1), (4, 7), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 1), (4, 6), (5, 6), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "o test:\n",
        "is\tsmall",
        "long",
        "uncated if",
        "nto the\tne",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 41, 2, 46);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 6), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 7), (5, 7), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "o test:\n",
        "is\tsmall",
        "long",
        "uncated if",
        "nto the\tne",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 40, 2, 46);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 6), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 7), (5, 7), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec![
        "o test:\n",
        "is\tsmall",
        "long",
        "uncated if",
        "nto the\tne",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 39, 2, 46);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 6), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 7), (5, 7), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-9
    {
      let expect = vec![
        "o test:\n",
        "is\tsmall",
        "long",
        "uncated if",
        "nto the\tne",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 38, 2, 46);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 6), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 7), (5, 7), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-10
    {
      let expect = vec![
        "o test:\n",
        "is\tsmall",
        "long",
        "uncated if",
        "nto the\tne",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 37, 2, 46);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 6), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 7), (5, 7), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-11
    {
      let expect = vec![
        "tains several thi",
        "the",
        "it\t",
        "3. The extra part",
        "4. The extra part",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 2, 2, 16);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 7), (4, 7), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 7), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-12
    {
      let expect = vec![
        "l it contains sev",
        "1. When\tth",
        "2. When\tit",
        "\t3. The ex",
        "\t4. The ex",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 1, 2, 8);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13
    {
      let expect = vec![
        "But still it cont",
        "\t1. When",
        "\t2. When",
        "\t\t3",
        "\t\t4",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 0, 2, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 0), (4, 0), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0), (3, 2), (4, 2), (5, 0), (6, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec!["", "", "", "enough\tto", "completely"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 35, 0, 68);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 1), (4, 6)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 1)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["", "", "", "enough\tto", "completely"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 29, 0, 68);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 1), (4, 6)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 1)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["", "", "", "enough\tto", "completely"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 28, 0, 68);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 1), (4, 6)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 1)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["", "", "", "enough\tto", "completely"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 27, 0, 68);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 1), (4, 6)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 1)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["", "", "", "\tenough", "to\tcomp"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 26, 0, 61);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 3)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec!["", "", "", "l\tenough", "to\tcom"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 25, 0, 60);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 4)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["", "", "", "ll\tenough", "to\tco"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 24, 0, 59);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 5)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 1), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec!["", "", "", "all\tenough", "to\tc"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 23, 0, 58);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 6)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec!["", "", "", "mall\tenoug", "to\t"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 22, 0, 57);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 7)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-9
    {
      let expect = vec!["", "", "", "small\tenou", "\tto"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 21, 0, 56);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 7)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-10
    {
      let expect = vec!["", "", "test:\n", "\tsmall", "long\tt"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 20, 0, 48);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 4)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 4), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-11, Center
    {
      let expect = vec!["", "", " test:\n", "s\tsmall", "long\t"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 19, 0, 47);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 5)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-12
    {
      let expect = vec!["", "", " test:\n", "s\tsmall", "long\t"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 20, 0, 47);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 5)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13
    {
      let expect = vec!["", "", " test:\n", "s\tsmall", "long\t"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 24, 0, 47);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 5)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
  #[test]
  fn new2_1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_nowrap();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "But still it cont",
        "\t1. When",
        "\t2. When",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 2)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-11, Center
    {
      let expect = vec![
        "",
        "l test lines.\n",
        "hings we want to ",
        "line\tis",
        "\ttoo",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 19, 0, 31);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 2)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 4)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-12
    {
      let expect = vec!["", "ines.\n", " want to test:\n", "is\t", "too\tlong"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 20, 0, 39);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 7), (4, 2)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13
    {
      let expect = vec!["", ".\n", "t to test:\n", "is\tsmal", "o\tlong"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 24, 0, 43);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 4)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-14
    {
      let expect = vec!["", "\n", " to test:\n", "is\tsmall", "\tlong"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 25, 0, 44);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 5)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-15
    {
      let expect = vec!["", "", ":\n", "small\t", "long\tto"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 26, 0, 52);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 4), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 3)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-16
    {
      let expect = vec!["", "", "\n", "small\te", "ong\tto"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 27, 0, 53);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 4)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-17
    {
      let expect = vec!["", "", "", "small\tenou", "\tto"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 30, 0, 56);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 7)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-18
    {
      let expect = vec!["", "", "", "mall\tenoug", "to\t"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 31, 0, 57);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 7)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-19
    {
      let expect = vec!["", "", "", "all\tenough", "to\tc"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 32, 0, 58);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 6)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-20
    {
      let expect = vec!["", "", "", "enough\t", "\tcompletel"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 33, 0, 66);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 3), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-21
    {
      let expect = vec!["", "", "", "enough\tt", "completely"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 34, 0, 67);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 2), (4, 7)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-22
    {
      let expect = vec!["", "", "", "enough\tto", "completely"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 35, 0, 68);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 1), (4, 6)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 1)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-23
    {
      let expect = vec!["", "", "", "to\t", "mpletely\tp"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 36, 0, 76);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 7), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-24
    {
      let expect = vec!["", "", "", "to\tc", "pletely\tpu"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 37, 0, 77);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 6), (4, 0)]
        .into_iter()
        .collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]
        .into_iter()
        .collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}

mod tests_search_anchor_horizontally_wrap_nolinebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into t",
        "he\tnext",
        "\trow,",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 0, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into t",
        "he\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 1, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into t",
        "he\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 3, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into t",
        "he\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 46, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into t",
        "he\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 47, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into t",
        "he\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 48, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 49, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 50, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 51, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-9
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 50, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-10
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 49, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-11
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 48, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-12
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 4, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 3, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-14
    {
      let expect = vec![
        "4. The extr",
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 2, 6, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-15
    {
      let expect = vec![
        "\t4. The ex",
        "tra parts are spl",
        "it into the",
        "\tnext",
        "\trow,",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 1, 6, 8);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-16
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into t",
        "he\tnext",
        "\trow,",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 0, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec!["the\t", "line\tis", "\tsmall", "\tenough", "\tto"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 35, 3, 17);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec!["the\t", "line\tis", "\tsmall", "\tenough", "\tto"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 10, 3, 17);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec!["the\t", "line\tis", "\tsmall", "\tenough", "\tto"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 9, 3, 17);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec!["\tthe", "\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 8, 3, 15);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["n\tthe", "\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 7, 3, 14);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec!["en\tthe", "\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 6, 3, 13);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec!["en\tthe", "\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 6, 3, 13);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec!["hen\tthe", "\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 5, 3, 12);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec!["When\tthe", "\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 4, 3, 11);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-9
    {
      let expect = vec![" When\tthe", "\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 3, 3, 10);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-10
    {
      let expect = vec![". When\tthe", "\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 2, 3, 9);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-11
    {
      let expect = vec!["1. When\tth", "e\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 1, 3, 8);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-12, Center
    {
      let expect = vec!["\t1. When", "\tthe", "\tline", "\tis", "\tsmall"];

      let actual = search_left_viewport(window.clone(), buf.clone(), 3, 0, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13
    {
      let expect = vec!["\t1. When", "\tthe", "\tline", "\tis", "\tsmall"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 24, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-14
    {
      let expect = vec!["\t1. When", "\tthe", "\tline", "\tis", "\tsmall"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 25, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
  #[test]
  fn new2_1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite s",
        "imple and small t",
        "est lines.\n",
        "But still it cont",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-12, Center
    {
      let expect = vec!["\t1. When", "\tthe", "\tline", "\tis", "\tsmall"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 0, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13
    {
      let expect = vec!["\t1. When", "\tthe", "\tline", "\tis", "\tsmall"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 24, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-14
    {
      let expect = vec!["\t1. When", "\tthe", "\tline", "\tis", "\tsmall"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 25, 3, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 4)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-15
    {
      let expect = vec!["1. When\t", "the\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 26, 3, 6);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 2)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-16
    {
      let expect = vec!["1. When\t", "the\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 27, 3, 6);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 2)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-18
    {
      let expect = vec!["1. When\t", "the\tline", "\tis", "\tsmall", "\tenough"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 32, 3, 6);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 2)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 3)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-19
    {
      let expect = vec!["the\t", "line\tis", "\tsmall", "\tenough", "\tto"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 33, 3, 17);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-20
    {
      let expect = vec!["the\t", "line\tis", "\tsmall", "\tenough", "\tto"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 34, 3, 17);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-21
    {
      let expect = vec!["the\t", "line\tis", "\tsmall", "\tenough", "\tto"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 35, 3, 17);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 6)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 7)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-22
    {
      let expect = vec!["line\t", "is\tsmall", "\tenough", "\tto", "\tcompletel"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 36, 3, 29);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 4);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_nolinebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "1. When\tthe line is\tsmall enough to completely put inside.\n",
        "2. The extra parts are been truncated if\tboth\tline-wrap\tand word-wrap options\tare\tnot\tset.\n",
        "3. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
        "4. The extra parts are been truncated if\tboth\tline-wrap\tand word-wrap options\tare\tnot\tset.\n",
        "5. When\tthe line is\tsmall enough to completely put inside.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "1. When\tth",
        "e line is\t",
        "small enough to c",
        "ompletely put ins",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "hen\tthe li",
        "ne is\tsmal",
        "l enough to compl",
        "etely put inside.",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 57, 0, 4);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "en\tthe lin",
        "e is\tsmall",
        " enough to comple",
        "tely put inside.\n",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 58, 0, 5);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "ncated if\t",
        "both\tline-",
        "wrap\tand w",
        "ord-wrap options",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 65, 1, 31);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 1)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3.1
    {
      let expect = vec!["and word-wra", "p options\t", "are\tnot", "\tset.\n"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 1, 85, 1, 72);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec!["and word-wra", "p options\t", "are\tnot", "\tset.\n"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 1, 90, 1, 72);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(1, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(1, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        1,
        2,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "e rows in the win",
        "dow, thus it may ",
        "contains less lin",
        "es in the buffer.",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 2, 299, 2, 309);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        " rows in the wind",
        "ow, thus it may c",
        "ontains less line",
        "s in the buffer.\n",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 2, 300, 2, 310);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        2,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec!["\tand word-", "wrap options", "\tare", "\tnot"];

      let actual = search_down_viewport(window.clone(), buf.clone(), 3, 55, 3, 69);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 6)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7.1
    {
      let expect = vec!["and word-wra", "p options\t", "are\tnot", "\tset.\n"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 85, 3, 72);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7.2
    {
      let expect = vec!["and word-wra", "p options\t", "are\tnot", "\tset.\n"];

      let actual = search_right_viewport(window.clone(), buf.clone(), 3, 96, 3, 72);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(3, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(3, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        3,
        4,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec![
        "en\tthe lin",
        "e is\tsmall",
        " enough to comple",
        "tely put inside.\n",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 4, 96, 4, 5);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8.1
    {
      let expect = vec![
        "hen\tthe li",
        "ne is\tsmal",
        "l enough to compl",
        "etely put inside.",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 4, 4, 4, 4);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(4, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        4,
        5,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}

mod tests_search_anchor_horizontally_wrap_linebreak {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 1, 5, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 1, 6, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 1, 13, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 1, 10, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 1, 2, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into ",
        "the\tnext",
        "\trow,",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 0, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into ",
        "the\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 1, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into ",
        "the\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 2, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into ",
        "the\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 47, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into ",
        "the\tnext",
        "\trow,",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 48, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "4. The extra",
        " parts are split ",
        "into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 49, 6, 11);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "4. The extra",
        " parts are split ",
        "into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 50, 6, 11);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "4. The extra",
        " parts are split ",
        "into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 51, 6, 11);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 5)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8, Center
    {
      let expect = vec![
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
        "\teither",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 52, 6, 27);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-9
    {
      let expect = vec![
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
        "\teither",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 15, 6, 27);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new2_1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 5);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "Hello, RSVIM!\n",
        "This is a quite simple and small test lines.\n",
        "But still it contains several things we want to test:\n",
        "\t1. When\tthe\tline\tis\tsmall\tenough\tto\tcompletely\tput\tinside.\n",
        "\t2. When\tit\t\ttoo\tlong\tto\tcompletely\tput:\n",
        "\t\t3. The extra parts are been truncated if\tboth\tline-wrap\tand\tword-wrap\toptions\tare\tnot\tset.\n",
        "\t\t4. The extra parts are split into the\tnext\trow,\tif\teither\tline-wrap\tor\tword-wrap\toptions\tare\tbeen\tset. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "Hello, RSVIM!\n",
        "This is a quite ",
        "simple and small ",
        "test lines.\n",
        "But still it ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> =
        vec![(0, 0), (1, 0), (2, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        3,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Prepare
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into ",
        "the\tnext",
        "\trow,",
      ];

      let actual = search_down_viewport(window.clone(), buf.clone(), 6, 0, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8, Center
    {
      let expect = vec![
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
        "\teither",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 6, 52, 6, 27);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-9
    {
      let expect = vec![
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
        "\teither",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 15, 6, 27);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-10
    {
      let expect = vec![
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
        "\teither",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 14, 6, 27);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-11
    {
      let expect = vec![
        "a parts are split",
        " into the\t",
        "next\trow,",
        "\tif",
        "\teither",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 13, 6, 27);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-12
    {
      let expect = vec![
        "extra parts are ",
        "split into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 9, 6, 23);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13
    {
      let expect = vec![
        " extra parts are ",
        "split into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 8, 6, 22);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-14
    {
      let expect = vec![
        "e extra parts are",
        " split into the",
        "\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 7, 6, 21);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-15
    {
      let expect = vec![
        "he extra parts ",
        "are split into ",
        "the\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 6, 6, 20);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-16
    {
      let expect = vec![
        "The extra parts ",
        "are split into ",
        "the\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 5, 6, 19);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-17
    {
      let expect = vec![
        " The extra parts ",
        "are split into ",
        "the\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 4, 6, 18);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-18
    {
      let expect = vec![
        ". The extra parts",
        " are split into ",
        "the\tnext",
        "\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 3, 6, 17);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-19
    {
      let expect = vec![
        "4. The extra ",
        "parts are split ",
        "into the\t",
        "next\trow,",
        "\tif",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 2, 6, 16);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-20
    {
      let expect = vec![
        "\t4. The ",
        "extra parts are ",
        "split into the",
        "\tnext",
        "\trow,",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 1, 6, 8);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-21
    {
      let expect = vec![
        "\t\t4",
        ". The extra parts",
        " are split into ",
        "the\tnext",
        "\trow,",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 6, 0, 6, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(6, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        6,
        7,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3() {
    test_log_init();

    let terminal_size = U16Size::new(17, 4);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "1. When the line contains some super long long word that cannot put, wewillhavetofallbacktonolinebreakbehaviorandthustrytogetmoresmoothbehavior thus to make a more smooth and eye friendly moving or scrolling behavior.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "1. When the line ",
        "contains some ",
        "super long long ",
        "word that cannot ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-1
    {
      let expect = vec![
        "1. When the line ",
        "contains some ",
        "super long long ",
        "word that cannot ",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 63, 0, 0);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-2
    {
      let expect = vec![
        "the line contains",
        " some super long ",
        "long word that ",
        "cannot put, ",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 68, 0, 8);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-3
    {
      let expect = vec![
        "ntains some super",
        " long long word ",
        "that cannot put, ",
        "wewillhavetofallb",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 69, 0, 19);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-4
    {
      let expect = vec![
        "ntains some super",
        " long long word ",
        "that cannot put, ",
        "wewillhavetofallb",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 85, 0, 19);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-5
    {
      let expect = vec![
        "r long long word ",
        "that cannot put, ",
        "wewillhavetofallb",
        "acktonolinebreakb",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 86, 0, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-6
    {
      let expect = vec![
        "r long long word ",
        "that cannot put, ",
        "wewillhavetofallb",
        "acktonolinebreakb",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 102, 0, 35);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-7
    {
      let expect = vec![
        "that cannot put, ",
        "wewillhavetofallb",
        "acktonolinebreakb",
        "ehaviorandthustry",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 103, 0, 52);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-8
    {
      let expect = vec![
        "that cannot put, ",
        "wewillhavetofallb",
        "acktonolinebreakb",
        "ehaviorandthustry",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 119, 0, 52);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-9
    {
      let expect = vec![
        "wewillhavetofallb",
        "acktonolinebreakb",
        "ehaviorandthustry",
        "togetmoresmoothbe",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 120, 0, 69);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-10
    {
      let expect = vec![
        "wewillhavetofallb",
        "acktonolinebreakb",
        "ehaviorandthustry",
        "togetmoresmoothbe",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 136, 0, 69);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-11
    {
      let expect = vec![
        "havetofallbackton",
        "olinebreakbehavio",
        "randthustrytogetm",
        "oresmoothbehavior",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 142, 0, 75);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-12
    {
      let expect = vec![
        "avetofallbacktono",
        "linebreakbehavior",
        "andthustrytogetmo",
        "resmoothbehavior ",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 143, 0, 76);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13, Center
    {
      let expect = vec![
        "ofallbacktonoline",
        "breakbehaviorandt",
        "hustrytogetmoresm",
        "oothbehavior thus",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 144, 0, 80);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-14
    {
      let expect = vec![
        "ofallbacktonoline",
        "breakbehaviorandt",
        "hustrytogetmoresm",
        "oothbehavior thus",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 0, 80, 0, 80);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }

  #[test]
  fn new3_1() {
    test_log_init();

    let terminal_size = U16Size::new(17, 4);
    let buf_opts = BufferLocalOptionsBuilder::default().build().unwrap();
    let win_opts = make_wrap_linebreak();

    let buf = make_buffer_from_lines(
      terminal_size,
      buf_opts,
      vec![
        "1. When the line contains some super long long word that cannot put, wewillhavetofallbacktonolinebreakbehaviorandthustrytogetmoresmoothbehavior thus to make a more smooth and eye friendly moving or scrolling behavior.\n",
      ],
    );

    let window = Rc::new(RefCell::new(make_window(
      terminal_size,
      buf.clone(),
      &win_opts,
    )));

    // Initialize
    {
      let expect = vec![
        "1. When the line ",
        "contains some ",
        "super long long ",
        "word that cannot ",
      ];

      let actual = window.borrow().viewport();
      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-13, Center
    {
      let expect = vec![
        "ofallbacktonoline",
        "breakbehaviorandt",
        "hustrytogetmoresm",
        "oothbehavior thus",
      ];

      let actual = search_right_viewport(window.clone(), buf.clone(), 0, 144, 0, 80);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-14
    {
      let expect = vec![
        "ofallbacktonoline",
        "breakbehaviorandt",
        "hustrytogetmoresm",
        "oothbehavior thus",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 0, 80, 0, 80);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-15
    {
      let expect = vec![
        "tofallbacktonolin",
        "ebreakbehaviorand",
        "thustrytogetmores",
        "moothbehavior ",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 0, 79, 0, 79);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-16
    {
      let expect = vec![
        "etofallbacktonoli",
        "nebreakbehavioran",
        "dthustrytogetmore",
        "smoothbehavior ",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 0, 78, 0, 78);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-17
    {
      let expect = vec![
        "wewillhavetofallb",
        "acktonolinebreakb",
        "ehaviorandthustry",
        "togetmoresmoothbe",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 0, 69, 0, 69);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-18
    {
      let expect = vec![
        " ",
        "wewillhavetofallb",
        "acktonolinebreakb",
        "ehaviorandthustry",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 0, 68, 0, 68);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-19
    {
      let expect = vec![
        ", ",
        "wewillhavetofallb",
        "acktonolinebreakb",
        "ehaviorandthustry",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 0, 67, 0, 67);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }

    // Search-20
    {
      let expect = vec![
        "put, ",
        "wewillhavetofallb",
        "acktonolinebreakb",
        "ehaviorandthustry",
      ];

      let actual = search_left_viewport(window.clone(), buf.clone(), 0, 64, 0, 64);

      let expect_start_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      let expect_end_fills: BTreeMap<usize, usize> = vec![(0, 0)].into_iter().collect();
      assert_viewport(
        buf.clone(),
        &actual,
        &expect,
        0,
        1,
        &expect_start_fills,
        &expect_end_fills,
      );
    }
  }
}
