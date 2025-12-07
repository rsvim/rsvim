#![allow(unused_imports)]

use crate::buf::BufferArc;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::Tree;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::content::Content;
use crate::ui::widget::window::opt::WindowOptions;
use std::sync::Arc;

pub fn make_window(
  terminal_size: U16Size,
  buffer: BufferArc,
  window_options: &WindowOptions,
) -> Window {
  let mut tree = Tree::new(terminal_size);
  tree.set_global_local_options(window_options);
  let window_shape = rect_from_size!(terminal_size, isize);
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
  let actual_shape = rect_from_size!(terminal_size, u16);
  let viewport = Viewport::view(
    &window_options,
    buffer.text(),
    &actual_shape,
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
  let mut tree = Tree::new(terminal_size);
  tree.set_global_local_options(&window_options);
  let shape = rect_from_size!(terminal_size, isize);
  let content =
    Content::new(shape, Arc::downgrade(&buffer), Arc::downgrade(&viewport));
  let mut canvas = Canvas::new(terminal_size);
  content.draw(&mut canvas);
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
