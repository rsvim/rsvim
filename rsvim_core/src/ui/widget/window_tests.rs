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
use crate::ui::canvas::Canvas;
use crate::ui::tree::Tree;
use crate::ui::tree::TreeNodeId;
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
use taffy::Style;

fn make_window_from_size(
  terminal_size: U16Size,
  buffer: BufferArc,
  window_options: WindowOptions,
) -> (Tree, TreeNodeId) {
  let mut tree = Tree::new(terminal_size).unwrap();
  tree.set_global_local_options(window_options);

  let window_style = Style {
    size: taffy::Size {
      width: taffy::Dimension::auto(),
      height: taffy::Dimension::auto(),
    },
    ..Default::default()
  };
  let window_id = tree
    .add_new_window(
      tree.root_id(),
      window_style,
      window_options,
      Arc::downgrade(&buffer),
    )
    .unwrap();
  (tree, window_id)
}

fn do_test_draw(actual: &Canvas, expect: &[&str]) {
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

  let window_local_options =
    WindowOptionsBuilder::default().wrap(false).build().unwrap();
  let (tree, window_id) =
    make_window_from_size(terminal_size, buf.clone(), window_local_options);
  let mut actual = Canvas::new(terminal_size);
  window.draw(&mut actual);
  do_test_draw(&actual, &expect);
}
