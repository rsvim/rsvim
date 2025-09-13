//! Vim buffer options.

pub mod file_encoding;
pub mod file_format;

use crate::defaults;
use derive_builder::Builder;
pub use file_encoding::*;
pub use file_format::*;

#[cfg(test)]
mod file_encoding_tests;
#[cfg(test)]
mod file_format_tests;

#[derive(Debug, Copy, Clone, Builder)]
/// Local buffer options.
pub struct BufferOptions {
  #[builder(default = defaults::buf::TAB_STOP)]
  tab_stop: u16,

  #[builder(default = defaults::buf::EXPAND_TAB)]
  expand_tab: bool,

  #[builder(default = defaults::buf::SHIFT_WIDTH)]
  shift_width: u16,

  #[builder(default = defaults::buf::FILE_ENCODING)]
  file_encoding: FileEncodingOption,

  #[builder(default = defaults::buf::FILE_FORMAT)]
  file_format: FileFormatOption,
}

impl BufferOptions {
  /// Buffer 'tab-stop' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27tabstop%27>.
  pub fn tab_stop(&self) -> u16 {
    self.tab_stop
  }

  pub fn set_tab_stop(&mut self, value: u16) {
    self.tab_stop = value;
  }

  /// Buffer 'expand-tab' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27expandtab%27>.
  pub fn expand_tab(&self) -> bool {
    self.expand_tab
  }

  pub fn set_expand_tab(&mut self, value: bool) {
    self.expand_tab = value;
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
