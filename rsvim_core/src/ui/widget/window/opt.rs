//! Window options.

use bitflags::bitflags;
use std::fmt::Debug;

pub const WRAP: bool = true;
pub const LINE_BREAK: bool = false;
pub const SCROLL_OFF: u8 = 0;

bitflags! {
  #[derive(Copy, Clone)]
  struct OptFlags: u8 {
    const WRAP = 1;
    const LINE_BREAK = 1 << 1;
  }
}

impl Debug for OptFlags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OptFlags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

#[allow(dead_code)]
// expand_tab
const OPT_FLAGS: OptFlags = OptFlags::WRAP;

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
/// Window local options.
pub struct WindowOptions {
  #[builder(default = WRAP)]
  wrap: bool,

  #[builder(default = LINE_BREAK)]
  line_break: bool,

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
