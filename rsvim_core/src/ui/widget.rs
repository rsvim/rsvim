//! Basic atom of all UI components.

pub mod cmdline;
pub mod cursor;
pub mod panel;
pub mod window;

#[cfg(test)]
mod window_tests;

use crate::buf::BufferManagerArc;
use crate::hl::ColorSchemeManagerArc;
use crate::syntax::SyntaxManagerArc;
use crate::ui::canvas::Canvas;

#[derive(Debug)]
pub struct WidgetContext {
  pub buffer_manager: BufferManagerArc,
  pub syntax_manager: BufferManagerArc,
  pub colorscheme_manager: ColorSchemeManagerArc,
}

impl WidgetContext {
  pub fn new(
    buffer_manager: BufferManagerArc,
    syntax_manager: SyntaxManagerArc,
    colorscheme_manager: ColorSchemeManagerArc,
  ) -> Self {
    Self {
      buffer_manager,
      syntax_manager,
      colorscheme_manager,
    }
  }
}

/// Base trait for all UI widgets.
pub trait Widgetable {
  /// Draw the widget to canvas, on the specific shape.
  fn draw(&self, _canvas: &mut Canvas, _context: &WidgetContext) {
    // Do nothing.
    // trace!("draw canvas");
  }
}

/// Generate enum dispatcher for `Widget`.
#[macro_export]
macro_rules! widgetable_enum_impl {
  ($enum:ident, $($variant:tt),*) => {
    impl Widgetable for $enum {
      fn draw(&self, canvas: &mut Canvas, context: &WidgetContext) {
        match self {
          $(
            $enum::$variant(w) => w.draw(canvas, context),
          )*
        }
      }
    }
  }
}
