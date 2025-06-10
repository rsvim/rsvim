//! Window local options.

use crate::defaults;

use derive_builder::Builder;

#[derive(Debug, Copy, Clone, Builder)]
/// Window local options.
pub struct WindowLocalOptions {
  #[builder(default = defaults::win::WRAP)]
  wrap: bool,

  #[builder(default = defaults::win::LINE_BREAK)]
  line_break: bool,

  #[builder(default = defaults::win::SCROLL_OFF)]
  scroll_off: u16,
}

impl WindowLocalOptions {
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
  pub fn scroll_off(&self) -> u16 {
    self.scroll_off
  }

  pub fn set_scroll_off(&mut self, value: u16) {
    self.scroll_off = value;
  }
}

#[derive(Debug, Copy, Clone, Builder)]
/// Global window options.
pub struct WindowGlobalOptions {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn options1() {
    let opt1 = WindowLocalOptionsBuilder::default()
      .wrap(true)
      .line_break(true)
      .scroll_off(3)
      .build()
      .unwrap();
    assert!(opt1.wrap());
    assert!(opt1.line_break());
    assert_eq!(opt1.scroll_off(), 3);

    let opt2 = WindowLocalOptionsBuilder::default().build().unwrap();
    assert_eq!(opt2.wrap(), defaults::win::WRAP);
    assert_eq!(opt2.line_break(), defaults::win::LINE_BREAK);
    assert_eq!(opt2.scroll_off(), defaults::win::SCROLL_OFF);
  }
}
