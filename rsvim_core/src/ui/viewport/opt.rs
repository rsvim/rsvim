//! Viewport options.

use crate::ui::widget::window::WindowLocalOptions;

#[derive(Debug, Copy, Clone)]
// Viewport options.
pub struct ViewportOptions {
  pub wrap: bool,
  pub line_break: bool,
  pub scroll_off: u16,
}

impl From<&WindowLocalOptions> for ViewportOptions {
  fn from(value: &WindowLocalOptions) -> Self {
    Self {
      wrap: value.wrap(),
      line_break: value.line_break(),
      scroll_off: value.scroll_off(),
    }
  }
}
