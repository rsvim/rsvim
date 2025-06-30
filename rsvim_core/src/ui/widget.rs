//! Basic atom of all UI components.

use crate::ui::canvas::Canvas;

// use tracing::trace;

pub mod command_line;
pub mod cursor;
pub mod root;
pub mod window;

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
macro_rules! widget_enum_dispatcher {
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
