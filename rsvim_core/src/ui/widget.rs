//! Basic atom of all UI components.

pub mod cmdline;
pub mod cursor;
pub mod panel;
pub mod window;

#[cfg(test)]
mod window_tests;

use crate::buf::BufferManagerArc;
use crate::ui::canvas::Canvas;

#[derive(Debug)]
pub struct WidgetDataAccess {
  pub buffer_manager: BufferManagerArc,
}

impl WidgetDataAccess {
  pub fn new(buffer_manager: BufferManagerArc) -> Self {
    Self { buffer_manager }
  }
}

/// Base trait for all UI widgets.
pub trait Widgetable {
  /// Draw the widget to canvas, on the specific shape.
  fn draw(&self, _canvas: &mut Canvas, _data_access: &WidgetDataAccess) {
    // Do nothing.
    // trace!("draw canvas");
  }
}

/// Generate enum dispatcher for `Widget`.
#[macro_export]
macro_rules! widgetable_enum_impl {
  ($enum:ident, $($variant:tt),*) => {
    impl Widgetable for $enum {
      fn draw(&self, canvas: &mut Canvas, data_access: &WidgetDataAccess) {
        match self {
          $(
            $enum::$variant(w) => w.draw(canvas, data_access),
          )*
        }
      }
    }
  }
}
