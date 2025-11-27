//! Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::Cell;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;
use compact_str::ToCompactString;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// The symbol for command-line indicator, i.e. the ':', '/', '?' char.
pub enum IndicatorSymbol {
  Empty,
  Ex,
  SearchForward,
  SearchBackward,
}

impl std::fmt::Display for IndicatorSymbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      IndicatorSymbol::Empty => write!(f, " "),
      IndicatorSymbol::Ex => write!(f, ":"),
      IndicatorSymbol::SearchForward => write!(f, "/"),
      IndicatorSymbol::SearchBackward => write!(f, "?"),
    }
  }
}

#[derive(Debug, Clone)]
/// Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.
pub struct CommandLineIndicator {
  base: InodeBase,
  symbol: IndicatorSymbol,
}

inode_impl!(CommandLineIndicator);

impl CommandLineIndicator {
  pub fn new(
    relationship: ItreeRc,
    id: TreeNodeId,
    symbol: IndicatorSymbol,
  ) -> Self {
    CommandLineIndicator {
      base: InodeBase::new(relationship, id),
      symbol,
    }
  }

  pub fn symbol(&self) -> IndicatorSymbol {
    self.symbol
  }

  pub fn set_symbol(&mut self, symbol: IndicatorSymbol) {
    self.symbol = symbol;
  }
}

impl Widgetable for CommandLineIndicator {
  fn draw(&self, canvas: &mut Canvas) {
    if self.visible() {
      let actual_shape = self.actual_shape();
      let upos: U16Pos = actual_shape.min().into();
      let symbol = self.symbol;
      let symbol = format!("{symbol}").to_compact_string();
      let cell = Cell::with_symbol(symbol);
      canvas.frame_mut().set_cell(upos, cell);
    }
  }
}
