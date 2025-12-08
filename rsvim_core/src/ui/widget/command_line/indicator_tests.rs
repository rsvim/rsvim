#![allow(unused_imports)]

use super::indicator::*;
use crate::buf::BufferArc;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::BufferOptionsBuilder;
use crate::prelude::*;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::buf::make_empty_buffer;
use crate::tests::log::init as test_log_init;
use crate::tests::viewport::assert_canvas;
use crate::ui::canvas::Canvas;
use crate::ui::tree::Tree;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use compact_str::ToCompactString;
use ropey::Rope;
use ropey::RopeBuilder;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::sync::Arc;

#[cfg(test)]
mod tests_nowrap {
  use super::*;

  fn make_canvas(
    terminal_size: U16Size,
    cmdline_indicator: &Indicator,
  ) -> Canvas {
    let mut canvas = Canvas::new(terminal_size);
    cmdline_indicator.draw(&mut canvas);
    canvas
  }

  #[test]
  fn new1() {
    test_log_init();

    let terminal_size = size!(1, 1);
    let terminal_shape = rect_from_size!(terminal_size);
    let terminal_shape = rect_as!(terminal_shape, isize);

    let expect = vec![":"];

    let cmdline_indicator = Indicator::new(terminal_shape, IndicatorSymbol::Ex);
    let actual = make_canvas(terminal_size, &cmdline_indicator);
    assert_canvas(&actual, &expect);
  }
}
