#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use crate::buf::BufferArc;
use crate::buf::text::Text;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::tree::Tree;
use crate::ui::tree::TreeArc;
use crate::ui::tree::TreeNode;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::opt::WindowOptions;
use std::sync::Arc;
use taffy::Style;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;

pub fn make_window(
  terminal_size: U16Size,
  buffer: BufferArc,
  window_options: &WindowOptions,
) -> Window {
  let mut tree = Tree::new(terminal_size);
  tree.set_global_local_options(window_options);
  let window_shape = rect_from_size!(terminal_size);
  let window_shape = rect_as!(window_shape, isize);
  Window::new(
    tree.global_local_options(),
    window_shape,
    Arc::downgrade(&buffer),
  )
}

pub fn make_viewport(
  terminal_size: U16Size,
  window_options: WindowOptions,
  buffer: BufferArc,
  start_line_idx: usize,
  start_column_idx: usize,
) -> ViewportArc {
  let buffer = lock!(buffer);
  let actual_shape = rect_from_size!(terminal_size);
  let actual_shape = rect_as!(actual_shape, u16);
  let actual_size = actual_shape.size();
  let viewport = Viewport::view(
    &window_options,
    buffer.text(),
    &actual_size,
    start_line_idx,
    start_column_idx,
  );
  Viewport::to_arc(viewport)
}

pub fn make_canvas(
  terminal_size: U16Size,
  window_options: WindowOptions,
  buffer: BufferArc,
  viewport: ViewportArc,
) -> Canvas {
  let mut tree = Tree::new(terminal_size).unwrap();
  tree.set_global_local_options(window_options);
  let shape = rect_from_size!(terminal_size);
  let shape = rect_as!(shape, isize);
  let style = Style {
    size: taffy::Size {
      height: taffy::Dimension::from_percent(1.0),
      width: taffy::Dimension::from_percent(1.0),
    },
    ..Default::default()
  };
  let window_id = tree
    .new_window_with_parent(
      tree.root_id(),
      style,
      *tree.global_local_options(),
      Arc::downgrade(&buffer),
    )
    .unwrap();
  let window_content_id = match tree.node(window_id).unwrap() {
    TreeNode::Window(window) => window.content_id(),
    _ => unreachable!(),
  };
  let mut canvas = Canvas::new(terminal_size);
  match tree.node(window_content_id).unwrap() {
    TreeNode::WindowContent(window_content) => content.draw(&mut canvas),
    _ => unreachable!(),
  }
  canvas
}

pub fn make_tree_canvas(tree: TreeArc, terminal_size: U16Size) -> CanvasArc {
  let canvas = Canvas::new(terminal_size);
  let canvas = Canvas::to_arc(canvas);
  let tree = lock!(tree);
  tree.draw(canvas.clone());
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

pub fn assert_viewport(
  text: &Text,
  actual: &Viewport,
  expect_rows: &Vec<&str>,
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
  for (i, e) in expect_rows.iter().enumerate() {
    info!("expect row[{}]:{:?}", i, e);
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
    let (first_line_idx, _first_line_viewport) =
      actual.lines().first().unwrap();
    let (last_line_idx, _last_line_viewport) = actual.lines().last().unwrap();
    assert_eq!(*first_line_idx, actual.start_line_idx());
    assert_eq!(*last_line_idx, actual.end_line_idx() - 1);
  }
  assert_eq!(
    actual.end_line_idx() - actual.start_line_idx(),
    actual.lines().len()
  );
  assert_eq!(
    actual.end_line_idx() - actual.start_line_idx(),
    expect_start_fills.len()
  );
  assert_eq!(
    actual.end_line_idx() - actual.start_line_idx(),
    expect_end_fills.len()
  );

  let buflines = text.rope().lines_at(actual.start_line_idx());
  let total_lines = expect_end_line - expect_start_line;

  for (l, line) in buflines.enumerate() {
    if l >= total_lines {
      break;
    }
    let actual_line_idx = l + expect_start_line;
    let line_viewport = actual.lines().get(&actual_line_idx).unwrap();

    info!(
      "l-{:?}, actual_line_idx:{}, line_viewport:{:?}",
      l, actual_line_idx, line_viewport
    );
    info!(
      "start_filled_cols expect:{:?}, actual:{}",
      expect_start_fills.get(&actual_line_idx),
      line_viewport.start_filled_cols()
    );
    assert_eq!(
      line_viewport.start_filled_cols(),
      *expect_start_fills.get(&actual_line_idx).unwrap()
    );
    info!(
      "end_filled_cols expect:{:?}, actual:{}",
      expect_end_fills.get(&actual_line_idx),
      line_viewport.end_filled_cols()
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
        r, payload, expect_rows[*r as usize]
      );
      assert_eq!(payload, expect_rows[*r as usize]);
    }
  }
}
