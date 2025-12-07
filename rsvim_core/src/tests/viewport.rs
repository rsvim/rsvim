use crate::buf::BufferArc;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::opt::FileFormatOption;
use crate::prelude::*;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::buf::make_empty_buffer;
use crate::tests::log::init as test_log_init;
use crate::ui::canvas::Canvas;
use crate::ui::tree::Tree;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::WindowOptions;
use crate::ui::widget::window::WindowOptionsBuilder;
use compact_str::ToCompactString;
use ropey::Rope;
use ropey::RopeBuilder;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::sync::Arc;

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
  let window_content =
    Content::new(shape, Arc::downgrade(&buffer), Arc::downgrade(&viewport));
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
