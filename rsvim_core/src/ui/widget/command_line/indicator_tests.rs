#![allow(unused_imports)]

use super::indicator::*;

use crate::buf::BufferArc;
use crate::buf::opt::{BufferLocalOptions, BufferLocalOptionsBuilder};
use crate::geo_size_as;
use crate::prelude::*;
use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
use crate::test::log::init as test_log_init;
use crate::ui::canvas::Canvas;
use crate::ui::tree::Tree;
use crate::ui::viewport::{Viewport, ViewportArc};
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::{
  WindowLocalOptions, WindowLocalOptionsBuilder,
};

use compact_str::ToCompactString;
use ropey::{Rope, RopeBuilder};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::sync::Arc;
use tracing::info;

#[cfg(test)]
mod tests_nowrap {
  use super::*;

  fn make_canvas(
    terminal_size: U16Size,
    cmdline_indicator: &CommandLineIndicator,
  ) -> Canvas {
    let mut canvas = Canvas::new(terminal_size);
    cmdline_indicator.draw(&mut canvas);
    canvas
  }

  fn assert_canvas(actual: &Canvas, expect: &[&str]) {
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
  fn new1() {
    test_log_init();

    let terminal_size = U16Size::new(1, 1);
    let terminal_shape = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );

    let expect = vec![":"];

    let cmdline_indicator =
      CommandLineIndicator::new(terminal_shape, CommandLineIndicatorSymbol::Ex);
    let actual = make_canvas(terminal_size, &cmdline_indicator);
    assert_canvas(&actual, &expect);
  }
}
