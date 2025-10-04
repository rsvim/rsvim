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
// wrap=true
// line_break=false
const OPT_FLAGS: OptFlags = OptFlags::WRAP;

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
/// Window local options.
pub struct WindowOptions {
  #[builder(default = OPT_FLAGS)]
  #[builder(setter(custom))]
  // wrap
  // line_break
  flags: OptFlags,

  #[builder(default = SCROLL_OFF)]
  scroll_off: u8,
}

impl WindowOptionsBuilder {
  pub fn wrap(&mut self, value: bool) -> &mut Self {
    let mut flags = match self.flags {
      Some(flags) => flags,
      None => OPT_FLAGS,
    };
    if value {
      flags.insert(OptFlags::WRAP);
    } else {
      flags.remove(OptFlags::WRAP);
    }
    self.flags = Some(flags);
    self
  }

  pub fn line_break(&mut self, value: bool) -> &mut Self {
    let mut flags = match self.flags {
      Some(flags) => flags,
      None => OPT_FLAGS,
    };
    if value {
      flags.insert(OptFlags::LINE_BREAK);
    } else {
      flags.remove(OptFlags::LINE_BREAK);
    }
    self.flags = Some(flags);
    self
  }
}

impl WindowOptions {
  /// The 'wrap' option, also known as 'line-wrap', default to `true`.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27wrap%27>.
  pub fn wrap(&self) -> bool {
    self.flags.contains(OptFlags::WRAP)
  }

  pub fn set_wrap(&mut self, value: bool) {
    if value {
      self.flags.insert(OptFlags::WRAP);
    } else {
      self.flags.remove(OptFlags::WRAP);
    }
  }

  /// The 'line-break' option, also known as 'word-wrap', default to `false`.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>.
  pub fn line_break(&self) -> bool {
    self.flags.contains(OptFlags::LINE_BREAK)
  }

  pub fn set_line_break(&mut self, value: bool) {
    if value {
      self.flags.insert(OptFlags::LINE_BREAK);
    } else {
      self.flags.remove(OptFlags::LINE_BREAK);
    }
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
