//! Basic atom of all UI components.

pub mod cmdline;
pub mod cursor;
pub mod panel;
pub mod window;

#[cfg(test)]
mod window_tests;

use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::TreeNodeId;
use crate::ui::viewport::CursorViewport;
use crate::ui::viewport::CursorViewportArc;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::window::opt::WindowOptions;

/// Base trait for all UI widgets.
pub trait Widgetable {
  /// Draw the widget to canvas, on the specific shape.
  fn draw(&self, _canvas: &mut Canvas) {
    // Do nothing.
    // trace!("draw canvas");
  }
}

/// Generate enum dispatcher for `Widget`.
#[macro_export]
macro_rules! widget_dispatcher {
  ($enum:ident, $($variant:tt),*) => {
    impl Widgetable for $enum {
      fn draw(&self, canvas: &mut Canvas) {
        match self {
          $(
            $enum::$variant(w) => w.draw(canvas),
          )*
        }
      }
    }
  }
}
