//! Global options for UI tree.

use crate::defaults;
use crate::ui::widget::window::WindowLocalOptions;

use regex::Regex;

#[derive(Debug, Clone)]
/// Global window options.
pub struct WindowGlobalOptions {
  break_at: String,
  break_at_regex: Regex,
}

impl WindowGlobalOptions {
  fn builder() -> WindowGlobalOptionsBuilder {
    WindowGlobalOptionsBuilder::default()
  }

  /// The 'break-at' option, default to `" ^I!@*-+;:,./?"`.
  /// See: <https://vimhelp.org/options.txt.html#%27breakat%27>.
  /// NOTE: This option represents the regex pattern to break word for 'line-break'.
  pub fn break_at(&self) -> &String {
    &self.break_at
  }

  // The build regex object for [`break_at`].
  pub fn break_at_regex(&self) -> &Regex {
    &self.break_at_regex
  }

  /// Set 'break-at' option.
  pub fn set_break_at(&mut self, value: &str) {
    self.break_at = String::from(value);
    self.break_at_regex = Regex::new(value).unwrap();
  }
}

#[derive(Debug, Clone)]
/// Global window options builder.
pub struct WindowGlobalOptionsBuilder {
  break_at: String,
}

impl WindowGlobalOptionsBuilder {
  pub fn break_at(&mut self, value: &str) -> &mut Self {
    self.break_at = String::from(value);
    self
  }
  pub fn build(&self) -> WindowGlobalOptions {
    WindowGlobalOptions {
      break_at: self.break_at.clone(),
      break_at_regex: Regex::new(&self.break_at).unwrap(),
    }
  }
}

impl Default for WindowGlobalOptionsBuilder {
  fn default() -> Self {
    WindowGlobalOptionsBuilder {
      // Defaults to `" ^I!@*-+;:,./?"`.
      break_at: String::from(defaults::win::BREAK_AT),
    }
  }
}

#[derive(Debug, Clone)]
/// Global options for UI tree.
pub struct GlobalOptions {
  /// Window local options.
  pub window_local_options: WindowLocalOptions,

  /// Window global options.
  pub window_global_options: WindowGlobalOptions,
}

impl Default for GlobalOptions {
  fn default() -> Self {
    GlobalOptions {
      window_local_options: WindowLocalOptions::builder().build(),
      window_global_options: WindowGlobalOptions::builder().build(),
    }
  }
}

#[cfg(test)]
mod tests {}
