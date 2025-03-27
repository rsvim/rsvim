//! Window local options.

use crate::defaults;

use derive_builder::Builder;

#[derive(Debug, Copy, Clone, Builder)]
/// Window local options.
pub struct WindowLocalOptions {
  #[builder(default = defaults::win::WRAP)]
  /// The 'wrap' option, also known as 'line-wrap', default to `true`.
  /// See: <https://vimhelp.org/options.txt.html#%27wrap%27>.
  pub wrap: bool,

  #[builder(default = defaults::win::LINE_BREAK)]
  /// The 'line-break' option, also known as 'word-wrap', default to `false`.
  /// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>.
  pub line_break: bool,

  #[builder(default = defaults::win::SCROLL_OFF)]
  /// The 'scroll-off' option, default to `0`.
  /// See: <https://vimhelp.org/options.txt.html#%27scrolloff%27>.
  pub scroll_off: u16,
}

#[derive(Debug, Copy, Clone, Builder)]
/// Global window options.
pub struct WindowGlobalOptions {}

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
      wrap: value.wrap,
      line_break: value.line_break,
      scroll_off: value.scroll_off,
    }
  }
}

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
    assert!(opt1.wrap);
    assert!(opt1.line_break);
    assert_eq!(opt1.scroll_off, 3);

    let opt2 = WindowLocalOptionsBuilder::default().build().unwrap();
    assert_eq!(opt2.wrap, defaults::win::WRAP);
    assert_eq!(opt2.line_break, defaults::win::LINE_BREAK);
    assert_eq!(opt2.scroll_off, defaults::win::SCROLL_OFF);
  }
}
