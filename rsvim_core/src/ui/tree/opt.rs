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

#[derive(Debug, Clone, Default)]
/// Global window options builder.
pub struct WindowGlobalOptionsBuilder {}

impl WindowGlobalOptionsBuilder {
  pub fn build(&self) -> WindowGlobalOptions {
    WindowGlobalOptions {}
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let _opt1 = WindowGlobalOptions::builder().build();
    let _opt2 = WindowGlobalOptionsBuilder::default().build();
  }
}
