//! Window local options.

use crate::defaults;

#[derive(Debug, Clone)]
/// Window local options.
pub struct LocalOptions {
  wrap: bool,
  line_break: bool,
}

impl Default for LocalOptions {
  fn default() -> Self {
    Self::builder().build()
  }
}

impl LocalOptions {
  pub fn builder() -> LocalOptionsBuilder {
    LocalOptionsBuilder::default()
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

/// The builder for [`LocalOptions`].
pub struct LocalOptionsBuilder {
  wrap: bool,
  line_break: bool,
}

impl LocalOptionsBuilder {
  pub fn wrap(&mut self, value: bool) -> &mut Self {
    self.wrap = value;
    self
  }
  pub fn line_break(&mut self, value: bool) -> &mut Self {
    self.line_break = value;
    self
  }
  pub fn build(&self) -> LocalOptions {
    LocalOptions {
      wrap: self.wrap,
      line_break: self.line_break,
    }
  }
}

impl Default for LocalOptionsBuilder {
  fn default() -> Self {
    LocalOptionsBuilder {
      wrap: defaults::win::WRAP,
      line_break: defaults::win::LINE_BREAK,
    }
  }
}

#[derive(Debug, Clone)]
/// Window global options.
pub struct GlobalOptions {}

impl Default for GlobalOptions {
  fn default() -> Self {
    Self::builder().build()
  }
}

impl GlobalOptions {
  pub fn builder() -> GlobalOptionsBuilder {
    GlobalOptionsBuilder::default()
  }
}

#[derive(Debug, Clone, Default)]
/// Window global options builder.
pub struct GlobalOptionsBuilder {}

impl GlobalOptionsBuilder {
  pub fn build(&self) -> GlobalOptions {
    GlobalOptions {}
  }
}

#[derive(Debug, Copy, Clone)]
// Viewport options.
pub struct ViewportOptions {
  pub wrap: bool,
  pub line_break: bool,
}

impl From<&LocalOptions> for ViewportOptions {
  fn from(value: &LocalOptions) -> Self {
    Self {
      wrap: value.wrap(),
      line_break: value.line_break(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn options1() {
    let mut builder = LocalOptionsBuilder::default();
    let opt1 = builder.wrap(true).line_break(true).build();
    assert!(opt1.wrap());
    assert!(opt1.line_break());

    let opt2 = LocalOptions::builder().build();
    assert!(opt2.wrap());
    assert!(!opt2.line_break());
  }
}
