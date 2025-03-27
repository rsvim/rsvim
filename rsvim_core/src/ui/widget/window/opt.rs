//! Window local options.

use crate::defaults;

#[derive(Debug, Copy, Clone)]
struct LocalOpts {
  pub wrap: bool,
  pub line_break: bool,
  pub scroll_off: u16,
}

impl Default for LocalOpts {
  fn default() -> Self {
    Self {
      wrap: defaults::win::WRAP,
      line_break: defaults::win::LINE_BREAK,
      scroll_off: defaults::win::SCROLL_OFF,
    }
  }
}

#[derive(Debug, Copy, Clone)]
/// Window options.
pub struct WindowLocalOptions {
  opts: LocalOpts,
}

impl Default for WindowLocalOptions {
  fn default() -> Self {
    Self::builder().build()
  }
}

impl WindowLocalOptions {
  pub fn builder() -> WindowOptionsBuilder {
    WindowOptionsBuilder::default()
  }

  /// The 'wrap' option, also known as 'line-wrap', default to `true`.
  /// See: <https://vimhelp.org/options.txt.html#%27wrap%27>.
  pub fn wrap(&self) -> bool {
    self.opts.wrap
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.opts.wrap = value;
  }

  /// The 'line-break' option, also known as 'word-wrap', default to `false`.
  /// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>.
  pub fn line_break(&self) -> bool {
    self.opts.line_break
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.opts.line_break = value;
  }

  /// The 'scroll-off' option, default to `0`.
  /// See: <https://vimhelp.org/options.txt.html#%27scrolloff%27>.
  pub fn scroll_off(&self) -> u16 {
    self.opts.scroll_off
  }

  pub fn set_scroll_off(&mut self, value: u16) {
    self.opts.scroll_off = value;
  }
}

#[derive(Debug, Copy, Clone)]
/// The builder for [`WindowLocalOptions`].
pub struct WindowOptionsBuilder {
  opts: LocalOpts,
}

impl WindowOptionsBuilder {
  pub fn wrap(&mut self, value: bool) -> &mut Self {
    self.opts.wrap = value;
    self
  }

  pub fn line_break(&mut self, value: bool) -> &mut Self {
    self.opts.line_break = value;
    self
  }

  pub fn scroll_off(&mut self, value: u16) -> &mut Self {
    self.opts.scroll_off = value;
    self
  }

  pub fn build(&self) -> WindowLocalOptions {
    WindowLocalOptions { opts: self.opts }
  }
}

impl Default for WindowOptionsBuilder {
  fn default() -> Self {
    WindowOptionsBuilder {
      opts: LocalOpts::default(),
    }
  }
}

#[derive(Debug, Clone)]
/// Global window options.
pub struct WindowGlobalOptions {}

impl Default for WindowGlobalOptions {
  fn default() -> Self {
    Self::builder().build()
  }
}

impl WindowGlobalOptions {
  pub fn builder() -> WindowGlobalOptionsBuilder {
    WindowGlobalOptionsBuilder::default()
  }
}

#[derive(Debug, Clone, Default)]
/// Global window options builder.
pub struct WindowGlobalOptionsBuilder {}

impl WindowGlobalOptionsBuilder {
  pub fn build(&self) -> WindowGlobalOptions {
    WindowGlobalOptions {}
  }
}

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn options1() {
    let mut builder = WindowOptionsBuilder::default();
    let opt1 = builder.wrap(true).line_break(true).build();
    assert!(opt1.wrap());
    assert!(opt1.line_break());

    let opt2 = WindowLocalOptions::builder().build();
    assert!(opt2.wrap());
    assert!(!opt2.line_break());
  }
}
