//! Global options for Vim windows.

#![allow(unused_imports)]

use crate::defaults;

use regex::Regex;

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

#[derive(Debug, Clone)]
/// Global window options builder.
pub struct WindowGlobalOptionsBuilder {}

impl WindowGlobalOptionsBuilder {
  pub fn build(&self) -> WindowGlobalOptions {
    WindowGlobalOptions {}
  }
}

impl Default for WindowGlobalOptionsBuilder {
  fn default() -> Self {
    WindowGlobalOptionsBuilder {}
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let _opt1 = WindowGlobalOptions::builder().build();
  }
}
