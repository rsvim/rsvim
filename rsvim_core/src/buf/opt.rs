//! Vim buffer options.

use crate::defaults;

use derive_builder::Builder;

// Re-export
pub use file_encoding::*;
pub use file_format::*;

pub mod file_encoding;
pub mod file_format;

#[derive(Debug, Copy, Clone, Builder)]
/// Local buffer options.
pub struct BufferLocalOptions {
  #[builder(default = defaults::buf::TAB_STOP)]
  tab_stop: u16,

  #[builder(default = defaults::buf::FILE_ENCODING)]
  file_encoding: FileEncodingOption,

  #[builder(default = defaults::buf::FILE_FORMAT)]
  file_format: FileFormatOption,
}

impl BufferLocalOptions {
  /// Buffer 'tab-stop' option.
  ///
  /// See: <https://vimhelp.org/options.txt.html#%27tabstop%27>.
  pub fn tab_stop(&self) -> u16 {
    self.tab_stop
  }

  pub fn set_tab_stop(&mut self, value: u16) {
    self.tab_stop = value;
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let opt1 = BufferLocalOptionsBuilder::default().build().unwrap();
    assert_eq!(opt1.tab_stop(), defaults::buf::TAB_STOP);
    assert_eq!(opt1.file_encoding(), defaults::buf::FILE_ENCODING);
  }
}
