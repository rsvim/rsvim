//! Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.

use crate::buf::unicode::char_is_whitespace;
use crate::inodify_impl;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::Cell;
use crate::ui::tree::*;
use crate::ui::widget::WidgetContext;
use crate::ui::widget::Widgetable;
use compact_str::ToCompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// The symbol for command-line indicator, i.e. the ':', '/', '?' char.
pub enum CmdlineIndicatorSymbol {
  Empty,
  Colon,
  Slash,
  Question,
}

impl std::fmt::Display for CmdlineIndicatorSymbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      CmdlineIndicatorSymbol::Empty => write!(f, " "),
      CmdlineIndicatorSymbol::Colon => write!(f, ":"),
      CmdlineIndicatorSymbol::Slash => write!(f, "/"),
      CmdlineIndicatorSymbol::Question => write!(f, "?"),
    }
  }
}

#[derive(Debug, Clone)]
/// Command-line indicator, i.e. the first char ':', '/', '?' in the commandline.
pub struct CmdlineIndicator {
  __node: InodeBase,
  symbol: CmdlineIndicatorSymbol,
}

inodify_impl!(CmdlineIndicator);

impl CmdlineIndicator {
  pub fn new(
    id: NodeId,
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
  fn draw(&self, canvas: &mut Canvas, context: &WidgetContext) {
    if self.enabled() {
      let actual_shape = self.actual_shape();
      let upos: U16Pos = actual_shape.min().into();
      let symbol = self.symbol;
      let symbol = format!("{symbol}").to_compact_string();
      let buffer_manager = lock!(context.buffer_manager);

      let mut cell = Cell::with_symbol(symbol);
      if let Some(colorscheme) = buffer_manager.colorscheme() {
        if cell.symbol().chars().any(char_is_whitespace) {
          cell.set_fg(Color::Reset);
        } else {
          cell.set_fg(colorscheme.ui_text());
        }
        cell.set_bg(colorscheme.ui_background());
        cell.set_attrs(Attributes::none());
      }

      canvas.frame_mut().set_cell(upos, cell);
    }
  }
}
