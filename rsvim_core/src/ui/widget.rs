//! Basic atom of all UI components.

pub mod cmdline;
pub mod cursor;
pub mod panel;
pub mod window;

#[cfg(test)]
mod window_tests;

use crate::ui::canvas::Canvas;

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
macro_rules! widgetable_enum_impl {
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
