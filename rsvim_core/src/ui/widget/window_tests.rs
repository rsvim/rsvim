#![allow(unused_imports)]

use super::window::*;
use crate::buf::Buffer;
use crate::buf::BufferArc;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::BufferOptionsBuilder;
use crate::prelude::*;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::buf::make_empty_buffer;
use crate::tests::log::init as test_log_init;
use crate::tests::viewport::assert_canvas;
use crate::tests::viewport::make_canvas;
use crate::tests::viewport::make_window;
use crate::ui::canvas::Canvas;
use crate::ui::tree::Tree;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::*;
use compact_str::ToCompactString;
use ropey::Rope;
use ropey::RopeBuilder;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::sync::Arc;
use std::sync::Once;

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

  let win_opts =
    WindowOptionsBuilder::default().wrap(false).build().unwrap();
  let mut actual = make_canvas(terminal_size, win_opts, buf.clone())
  assert_canvas(&actual, &expect);
}
