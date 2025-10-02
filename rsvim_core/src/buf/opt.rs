//! Vim buffer options.

pub mod file_encoding;
pub mod file_format;

#[cfg(test)]
mod file_encoding_tests;
#[cfg(test)]
mod file_format_tests;

use std::fmt::Debug;

use bitflags::bitflags;
pub use file_encoding::*;
pub use file_format::*;

bitflags! {
  #[derive(Copy, Clone)]
  pub struct BufferOptionFlags :u8 {
    const EXPAND_TAB = 1;
  }
}

impl Debug for BufferOptionFlags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BufferOptionFlags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

/// Buffer default options.
pub const TAB_STOP: u8 = 8;
pub const EXPAND_TAB: bool = false;
pub const BUFFER_OPTION_FLAGS: BufferOptionFlags =
  BufferOptionFlags::EXPAND_TAB;
pub const SHIFT_WIDTH: u16 = 8;
pub const FILE_ENCODING: FileEncodingOption = FileEncodingOption::Utf8;
#[cfg(target_os = "windows")]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Dos;
#[cfg(not(target_os = "windows"))]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Unix;

#[derive(Debug, Copy, Clone, derive_builder::Builder)]
/// Local buffer options.
pub struct BufferOptions {
  #[builder(default = TAB_STOP)]
  tab_stop: u8,

  // expand_tab
  #[builder(default = BUFFER_OPTION_FLAGS)]
  flags: BufferOptionFlags,

  #[builder(default = SHIFT_WIDTH)]
  shift_width: u16,

  #[builder(default = FILE_ENCODING)]
  file_encoding: FileEncodingOption,

  #[builder(default = FILE_FORMAT)]
  file_format: FileFormatOption,
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
    self.flags.contains(BufferOptionFlags::EXPAND_TAB)
  }

  pub fn set_expand_tab(&mut self, value: bool) {
    if value {
      self.flags.insert(BufferOptionFlags::EXPAND_TAB)
    } else {
      self.flags.remove(BufferOptionFlags::EXPAND_TAB)
    }
  }

  /// Buffer 'shift-width' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27shiftwidth%27>.
  pub fn shift_width(&self) -> u16 {
    self.shift_width
  }

  pub fn set_shift_width(&mut self, value: u16) {
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
