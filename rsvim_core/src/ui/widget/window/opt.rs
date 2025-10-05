//! Window options.

use crate::flags_builder_impl;
use crate::flags_impl;

flags_impl!(Flags, u8, WRAP, wrap, 1, LINE_BREAK, line_break, 1 << 1);

pub const WRAP: bool = true;
pub const LINE_BREAK: bool = false;
pub const SCROLL_OFF: u8 = 0;

// wrap=true
// line_break=false
const FLAGS: Flags = Flags::WRAP;

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
/// Window local options.
pub struct WindowOptions {
  #[builder(default = FLAGS)]
  #[builder(setter(custom))]
  // wrap=true
  // line_break=false
  flags: Flags,

  #[builder(default = SCROLL_OFF)]
  scroll_off: u8,
}

impl WindowOptions {
  /// The 'wrap' option, also known as 'line-wrap', default to `true`.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27wrap%27>.
  pub fn wrap(&self) -> bool {
    self.wrap
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.wrap = value;
  }

  /// The 'line-break' option, also known as 'word-wrap', default to `false`.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>.
  pub fn line_break(&self) -> bool {
    self.line_break
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.line_break = value;
  }

  /// The 'scroll-off' option, default to `0`.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27scrolloff%27>.
  pub fn scroll_off(&self) -> u8 {
    self.scroll_off
  }

  pub fn set_scroll_off(&mut self, value: u8) {
    self.scroll_off = value;
  }
}

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
/// Global window options.
pub struct WindowGlobalOptions {}
