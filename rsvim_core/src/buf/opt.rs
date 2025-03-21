//! Vim buffer options.

use crate::defaults;

// Re-export
pub use file_encoding::FileEncodingOption;

pub mod file_encoding;

#[derive(Debug, Clone)]
/// Local buffer options.
pub struct BufferLocalOptions {
  tab_stop: u16,
  file_encoding: FileEncodingOption,
}

impl Default for BufferLocalOptions {
  fn default() -> Self {
    Self::builder().build()
  }
}

impl BufferLocalOptions {
  pub fn builder() -> BufferLocalOptionsBuilder {
    BufferLocalOptionsBuilder::default()
  }

  pub fn tab_stop(&self) -> u16 {
    self.tab_stop
  }

  pub fn set_tab_stop(&mut self, value: u16) {
    self.tab_stop = value;
  }

  pub fn file_encoding(&self) -> FileEncodingOption {
    self.file_encoding
  }

  pub fn set_file_encoding(&mut self, value: FileEncodingOption) {
    self.file_encoding = value;
  }
}

#[derive(Debug, Clone)]
/// Local buffer options builder.
pub struct BufferLocalOptionsBuilder {
  tab_stop: u16,
  file_encoding: FileEncodingOption,
}

impl BufferLocalOptionsBuilder {
  pub fn tab_stop(&mut self, value: u16) -> &mut Self {
    self.tab_stop = value;
    self
  }

  pub fn file_encoding(&mut self, value: FileEncodingOption) -> &mut Self {
    self.file_encoding = value;
    self
  }

  pub fn build(&self) -> BufferLocalOptions {
    BufferLocalOptions {
      tab_stop: self.tab_stop,
      file_encoding: self.file_encoding,
    }
  }
}

impl Default for BufferLocalOptionsBuilder {
  fn default() -> Self {
    BufferLocalOptionsBuilder {
      tab_stop: defaults::buf::TAB_STOP,
      file_encoding: defaults::buf::FILE_ENCODING,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let opt1 = BufferLocalOptions::default();
    let opt2 = BufferLocalOptionsBuilder::default().build();
    assert_eq!(opt1.tab_stop(), opt2.tab_stop());
  }
}
