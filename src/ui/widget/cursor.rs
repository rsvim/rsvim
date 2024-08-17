//! Cursor widget.

use std::fmt::Debug;
use std::time::Duration;
use tracing::debug;

use crate::cart::{U16Pos, U16Rect};
use crate::glovar;
use crate::ui::canvas::{frame, CanvasArc, CursorStyle, CursorStyleFormatter};
use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

#[derive(Clone, Copy)]
/// Cursor widget.
pub struct Cursor {
  id: WidgetId,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new() -> Self {
    Cursor {
      id: uuid::next(),
      blinking: true,
      hidden: false,
      style: CursorStyle::DefaultUserShape,
    }
  }
}

impl Default for Cursor {
  fn default() -> Self {
    Cursor::new()
  }
}

impl Debug for Cursor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let style_formatter = CursorStyleFormatter::from(self.style);
    f.debug_struct("Cursor")
      .field("id", &self.id)
      .field("blinking", &self.blinking)
      .field("hidden", &self.hidden)
      .field("style", &style_formatter)
      .finish()
  }
}

impl Widget for Cursor {
  fn id(&self) -> WidgetId {
    self.id
  }

  fn draw(&mut self, actual_shape: U16Rect, terminal: CanvasArc) {
    let pos: U16Pos = actual_shape.min().into();
    debug!(
      "draw, actual shape:{:?}, top-left pos:{:?}",
      actual_shape, pos
    );

    terminal
      .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap()
      .frame_mut()
      .set_cursor(frame::Cursor::new(
        pos,
        self.blinking,
        self.hidden,
        self.style,
      ));
  }
}
