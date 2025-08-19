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
pub enum IndicatorSymbol {
  Empty,
  Ex,
  SearchForward,
  SearchBackard,
}

impl std::fmt::Display for IndicatorSymbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      IndicatorSymbol::Empty => write!(f, " "),
      IndicatorSymbol::Ex => write!(f, ":"),
      IndicatorSymbol::SearchForward => write!(f, "/"),
      IndicatorSymbol::SearchBackard => write!(f, "?"),
    }
  }
}

#[derive(Debug, Clone)]
/// Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.
pub struct Indicator {
  base: InodeBase,
  symbol: IndicatorSymbol,
}

impl Indicator {
  pub fn new(shape: IRect, symbol: IndicatorSymbol) -> Self {
    let base = InodeBase::new(shape);
    Indicator { base, symbol }
  }

  pub fn symbol(&self) -> IndicatorSymbol {
    self.symbol
  }

  pub fn set_symbol(&mut self, symbol: IndicatorSymbol) {
    self.symbol = symbol;
  }
}

inode_impl!(Indicator, base);

impl Widgetable for Indicator {
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
