//! Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.

use crate::inodify;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::Cell;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;
use compact_str::ToCompactString;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// The symbol for command-line indicator, i.e. the ':', '/', '?' char.
pub enum CmdlineIndicatorSymbol {
  Empty,
  Ex,
  SearchForward,
  SearchBackward,
}

impl std::fmt::Display for CmdlineIndicatorSymbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      CmdlineIndicatorSymbol::Empty => write!(f, " "),
      CmdlineIndicatorSymbol::Ex => write!(f, ":"),
      CmdlineIndicatorSymbol::SearchForward => write!(f, "/"),
      CmdlineIndicatorSymbol::SearchBackward => write!(f, "?"),
    }
  }
}

#[derive(Debug, Clone)]
/// Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.
pub struct CmdlineIndicator {
  __node: InodeBase,
  symbol: CmdlineIndicatorSymbol,
}

inodify!(CmdlineIndicator);

impl CmdlineIndicator {
  pub fn new(
    id: TreeNodeId,
    ctx: TreeContextWk,
    symbol: CmdlineIndicatorSymbol,
  ) -> Self {
    CmdlineIndicator {
      __node: InodeBase::new(id, ctx),
      symbol,
    }
  }

  pub fn symbol(&self) -> CmdlineIndicatorSymbol {
    self.symbol
  }

  pub fn set_symbol(&mut self, symbol: CmdlineIndicatorSymbol) {
    self.symbol = symbol;
  }
}

impl Widgetable for CmdlineIndicator {
  fn draw(&self, canvas: &mut Canvas) {
    if self.enabled() {
      let actual_shape = self.actual_shape();
      let upos: U16Pos = actual_shape.min().into();
      let symbol = self.symbol;
      let symbol = format!("{symbol}").to_compact_string();
      let cell = Cell::with_symbol(symbol);
      canvas.frame_mut().set_cell(upos, cell);
    }
  }
}
