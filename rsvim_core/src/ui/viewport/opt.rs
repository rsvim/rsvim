//! Viewport options.

use crate::ui::widget::window::WindowLocalOptions;

#[derive(Debug, Copy, Clone)]
/// Viewport options.
pub struct ViewportOptions {
  wrap: bool,
  line_break: bool,
  scroll_off: u16,
}

impl ViewportOptions {
  pub fn wrap(&self) -> bool {
    self.wrap
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.wrap = value;
  }

  pub fn line_break(&self) -> bool {
    self.line_break
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.line_break = value;
  }

  pub fn scroll_off(&self) -> u16 {
    self.scroll_off
  }

  pub fn set_scroll_off(&mut self, value: u16) {
    self.scroll_off = value;
  }
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
