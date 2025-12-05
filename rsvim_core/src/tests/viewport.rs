#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

//! Viewport, window, editing related test utils.

use crate::buf::BufferArc;
use crate::buf::text::Text;
use crate::coord::U16Size;
use crate::prelude::*;
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::{Tree, TreeArc};
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::WindowOptions;
use log::info;
use std::sync::Arc;
use taffy::Style;
use taffy::prelude::FromPercent;
use taffy::prelude::TaffyAuto;

pub fn make_canvas(
  terminal_size: U16Size,
  window_options: WindowOptions,
  buffer: BufferArc,
  viewport: ViewportArc,
) -> (Tree, Canvas) {
  let mut tree = Tree::new(terminal_size).unwrap();
  let tree_root_id = tree.root_id();
  let window_id = tree
    .add_new_window(
      tree_root_id,
      Style {
        size: taffy::Size {
          height: taffy::Dimension::from_percent(1.0),
          width: taffy::Dimension::from_percent(1.0),
        },
        ..Default::default()
      },
      window_options,
      Arc::downgrade(&buffer),
    )
    .unwrap();
  tree.set_window_viewport(window_id, viewport);
  let window_content = tree.window_content(window_id).unwrap();
  let mut canvas = Canvas::new(terminal_size);
  window_content.draw(&mut canvas);
  (tree, canvas)
}

pub fn make_canvas_from_tree(
  tree: TreeArc,
  terminal_size: U16Size,
) -> CanvasArc {
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

pub fn make_viewport(
  terminal_size: U16Size,
  window_options: WindowOptions,
  buffer: BufferArc,
  start_line_idx: usize,
  start_column_idx: usize,
) -> ViewportArc {
  let buffer = lock!(buffer);
  let actual_shape = rect_from_size!(terminal_size);
  let viewport = Viewport::view(
    &window_options,
    buffer.text(),
    &actual_shape,
    start_line_idx,
    start_column_idx,
  );
  Viewport::to_arc(viewport)
}

pub fn make_window(
  terminal_size: U16Size,
  buffer: BufferArc,
  window_options: WindowOptions,
) -> (Tree, TreeNodeId, ViewportArc) {
  let mut tree = Tree::new(terminal_size).unwrap();
  tree.set_global_local_options(window_options);
  let window_style = Style {
    size: taffy::Size {
      height: taffy::Dimension::from_percent(1.0),
      width: taffy::Dimension::from_percent(1.0),
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
  let viewport = tree.window(window_id).unwrap().viewport();
  (tree, window_id, viewport)
}

#[allow(clippy::too_many_arguments)]
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
