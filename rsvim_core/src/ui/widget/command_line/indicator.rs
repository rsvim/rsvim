//! Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

use compact_str::ToCompactString;
use geo::point;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// The symbol for command-line indicator, i.e. the ':', '/', '?' char.
pub enum CommandLineIndicatorSymbol {
  Empty,
  Ex,
  SearchForward,
  SearchBackard,
}

impl std::fmt::Display for CommandLineIndicatorSymbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      CommandLineIndicatorSymbol::Empty => write!(f, " "),
      CommandLineIndicatorSymbol::Ex => write!(f, ":"),
      CommandLineIndicatorSymbol::SearchForward => write!(f, "/"),
      CommandLineIndicatorSymbol::SearchBackard => write!(f, "?"),
    }
  }
}

#[derive(Debug, Clone)]
/// Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.
pub struct CommandLineIndicator {
  base: InodeBase,
  symbol: CommandLineIndicatorSymbol,
}

impl CommandLineIndicator {
  pub fn new(shape: IRect, symbol: CommandLineIndicatorSymbol) -> Self {
    let base = InodeBase::new(shape);
    CommandLineIndicator { base, symbol }
  }

  pub fn symbol(&self) -> CommandLineIndicatorSymbol {
    self.symbol
  }

  pub fn set_symbol(&mut self, symbol: CommandLineIndicatorSymbol) {
    self.symbol = symbol;
  }
}

inode_impl!(CommandLineIndicator, base);

impl Widgetable for CommandLineIndicator {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let upos: U16Pos = actual_shape.min().into();
    let symbol = self.symbol;
    let symbol = format!("{symbol}").to_compact_string();
    let cell = Cell::with_symbol(symbol);
    let cell_upos = point!(x: upos.x(), y: upos.y());
    canvas.frame_mut().set_cell(cell_upos, cell);
  }
}

#[allow(unused_imports)]
#[cfg(test)]
mod tests_nowrap {
  use super::*;

  use crate::buf::BufferArc;
  use crate::buf::opt::{BufferLocalOptions, BufferLocalOptionsBuilder};
  use crate::geo_size_as;
  use crate::prelude::*;
  use crate::test::buf::{make_buffer_from_lines, make_empty_buffer};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::Tree;
  use crate::ui::viewport::{Viewport, ViewportArc};
  use crate::ui::widget::window::{WindowLocalOptions, WindowLocalOptionsBuilder};

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use tracing::info;

  fn make_canvas(terminal_size: U16Size, cmdline_indicator: &CommandLineIndicator) -> Canvas {
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
