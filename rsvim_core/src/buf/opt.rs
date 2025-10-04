//! Vim buffer options.

pub mod file_encoding;
pub mod file_format;

#[cfg(test)]
mod file_encoding_tests;
#[cfg(test)]
mod file_format_tests;

use bitflags::bitflags;
pub use file_encoding::*;
pub use file_format::*;
use std::fmt::Debug;

// Buffer default options.
pub const TAB_STOP: u8 = 8;
pub const EXPAND_TAB: bool = false;
pub const SHIFT_WIDTH: u8 = 8;
pub const FILE_ENCODING: FileEncodingOption = FileEncodingOption::Utf8;
#[cfg(target_os = "windows")]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Dos;
#[cfg(not(target_os = "windows"))]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Unix;

bitflags! {
  #[derive(Copy, Clone)]
  struct OptFlags: u8 {
    const EXPAND_TAB = 1;
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
// expand_tab=false
const OPT_FLAGS: OptFlags = OptFlags::empty();

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
/// Local buffer options.
pub struct BufferOptions {
  #[builder(default = TAB_STOP)]
  tab_stop: u8,

  #[builder(default = OPT_FLAGS)]
  #[builder(setter(custom))]
  // expand_tab
  flags: OptFlags,

  #[builder(default = SHIFT_WIDTH)]
  shift_width: u8,

  #[builder(default = FILE_ENCODING)]
  file_encoding: FileEncodingOption,

  #[builder(default = FILE_FORMAT)]
  file_format: FileFormatOption,
}

impl BufferOptionsBuilder {
  #[allow(dead_code)]
  pub fn expand_tab(&mut self, value: bool) -> &mut Self {
    let mut flags = match self.flags {
      Some(flags) => flags,
      None => OPT_FLAGS,
    };
    if value {
      flags.insert(OptFlags::EXPAND_TAB);
    } else {
      flags.remove(OptFlags::EXPAND_TAB);
    }
    self.flags = Some(flags);
    self
  }
}

impl BufferOptions {
  /// Buffer 'tab-stop' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27tabstop%27>.
  pub fn tab_stop(&self) -> u8 {
    self.tab_stop
  }

  pub fn set_tab_stop(&mut self, value: u8) {
    self.tab_stop = value;
  }

  /// Buffer 'expand-tab' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27expandtab%27>.
  pub fn expand_tab(&self) -> bool {
    self.flags.contains(OptFlags::EXPAND_TAB)
  }

  pub fn set_expand_tab(&mut self, value: bool) {
    if value {
      self.flags.insert(OptFlags::EXPAND_TAB);
    } else {
      self.flags.remove(OptFlags::EXPAND_TAB);
    }
  }

  /// Buffer 'shift-width' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27shiftwidth%27>.
  pub fn shift_width(&self) -> u8 {
    self.shift_width
  }

  pub fn set_shift_width(&mut self, value: u8) {
    self.shift_width = value;
  }

  /// Buffer 'file-encoding' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27fileencoding%27>.
  pub fn file_encoding(&self) -> FileEncodingOption {
    self.file_encoding
  }

  pub fn set_file_encoding(&mut self, value: FileEncodingOption) {
    self.file_encoding = value;
  }

  /// Buffer 'file-format' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27fileformat%27>.
  pub fn file_format(&self) -> FileFormatOption {
    self.file_format
  }

  pub fn set_file_format(&mut self, value: FileFormatOption) {
    self.file_format = value;
  }

  /// Get 'end-of-line' based on 'file-format' option.
  pub fn end_of_line(&self) -> EndOfLineOption {
    self.file_format.into()
  }
}
