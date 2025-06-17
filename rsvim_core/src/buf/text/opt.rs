//! Text options.

use crate::buf::opt::{EndOfLineOption, FileEncodingOption, FileFormatOption};

#[derive(Debug, Copy, Clone)]
/// Text options.
pub struct TextOptions {
  tab_stop: u16,
  file_encoding: FileEncodingOption,
  file_format: FileFormatOption,
}

impl TextOptions {
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

  pub fn file_format(&self) -> FileFormatOption {
    self.file_format
  }

  pub fn set_file_format(&mut self, value: FileFormatOption) {
    self.file_format = value;
  }

  pub fn end_of_line(&self) -> EndOfLineOption {
    self.file_format.into()
  }
}
