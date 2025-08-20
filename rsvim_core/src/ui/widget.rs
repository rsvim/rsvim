//! Basic atom of all UI components.

use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::TreeNodeId;
use crate::ui::viewport::{CursorViewportArc, ViewportArc};
use crate::ui::widget::window::opt::WindowOptions;

pub mod command_line;
pub mod cursor;
pub mod root;
pub mod window;

#[cfg(test)]
mod window_tests;

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

pub trait EditableWidgetable {
  fn editable_viewport(&self) -> ViewportArc;

  fn set_editable_viewport(&mut self, viewport: ViewportArc);

  fn editable_cursor_viewport(&self) -> CursorViewportArc;

  fn set_editable_cursor_viewport(
    &mut self,
    cursor_viewport: CursorViewportArc,
  );

  fn editable_options(&self) -> &WindowOptions;

  fn editable_actual_shape(&self) -> &U16Rect;

  fn move_editable_cursor_to(&mut self, x: isize, y: isize) -> Option<IRect>;

  fn editable_cursor_id(&self) -> Option<TreeNodeId>;
}
