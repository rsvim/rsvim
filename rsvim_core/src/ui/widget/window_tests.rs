#![allow(unused_imports)]

use crate::buf::opt::BufferOptionsBuilder;
use crate::prelude::*;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::log::init as test_log_init;
use crate::tests::viewport::assert_canvas;
use crate::tests::viewport::make_canvas;
use crate::tests::viewport::make_window;
use crate::ui::widget::window::opt::*;

#[test]
fn draw_after_init1() {
  test_log_init();

  let terminal_size = size!(10, 10);
  let buf_opts = BufferOptionsBuilder::default().build().unwrap();
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
    "          ",
    "          ",
    "          ",
  ];

  let window_opts =
    WindowOptionsBuilder::default().wrap(false).build().unwrap();
  let (tree, window_id, _viewport) =
    make_window(terminal_size, buf.clone(), window_opts);
  let (_tree, actual) = make_canvas(
    terminal_size,
    window_opts,
    buf.clone(),
    tree.window(window_id).unwrap().viewport(),
  );
  assert_canvas(&actual, &expect);
}
