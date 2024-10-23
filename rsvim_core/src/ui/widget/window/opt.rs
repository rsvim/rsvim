//! Window local options.

use crate::defaults;

#[derive(Debug, Clone)]
/// Window options.
pub struct WindowLocalOptions {
  wrap: bool,
  line_break: bool,
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
    self.wrap
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.wrap = value;
  }

  /// The 'line-break' option, also known as 'word-wrap', default to `false`.
  /// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>.
  pub fn line_break(&self) -> bool {
    self.line_break
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.line_break = value;
  }
}

/// The builder for [`WindowLocalOptions`].
pub struct WindowOptionsBuilder {
  wrap: bool,
  line_break: bool,
}

impl WindowOptionsBuilder {
  pub fn wrap(&mut self, value: bool) -> &mut Self {
    self.wrap = value;
    self
  }
  pub fn line_break(&mut self, value: bool) -> &mut Self {
    self.line_break = value;
    self
  }
  pub fn build(&self) -> WindowLocalOptions {
    WindowLocalOptions {
      wrap: self.wrap,
      line_break: self.line_break,
    }
  }
}

impl Default for WindowOptionsBuilder {
  fn default() -> Self {
    WindowOptionsBuilder {
      wrap: defaults::win::WRAP,
      line_break: defaults::win::LINE_BREAK,
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
