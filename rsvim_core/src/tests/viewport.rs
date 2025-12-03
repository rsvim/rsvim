#![allow(unused_imports)]

//! Viewport, window, editing related test utils.

use crate::buf::BufferArc;
use crate::coord::U16Size;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::Tree;
use crate::ui::tree::TreeNodeId;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::Widgetable;
use log::info;
use std::sync::Arc;
use taffy::prelude::FromPercent;
use taffy::Style;

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
) -> (Tree, TreeNodeId) {
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
  (tree, window_id)
}
