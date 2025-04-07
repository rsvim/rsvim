//! Vim buffer options.

use crate::defaults;

use derive_builder::Builder;

// Re-export
pub use file_encoding::*;

pub mod file_encoding;

#[derive(Debug, Copy, Clone, Builder)]
/// Local buffer options.
pub struct BufferLocalOptions {
  #[builder(default = defaults::buf::TAB_STOP)]
  tab_stop: u16,

  #[builder(default = defaults::buf::FILE_ENCODING)]
  file_encoding: FileEncodingOption,
}

impl BufferLocalOptions {
  /// Buffer 'tab-stop' option.
  /// See: <https://vimhelp.org/options.txt.html#%27tabstop%27>.
  pub fn tab_stop(&self) -> u16 {
    self.tab_stop
  }

  pub fn set_tab_stop(&mut self, value: u16) {
    self.tab_stop = value;
  }

  /// Buffer 'file-encoding' option.
  /// See: <https://vimhelp.org/options.txt.html#%27fileencoding%27>.
  pub fn file_encoding(&self) -> FileEncodingOption {
    self.file_encoding
  }

  pub fn set_file_encoding(&mut self, value: FileEncodingOption) {
    self.file_encoding = value;
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
