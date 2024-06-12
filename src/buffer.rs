//! The VIM buffer.

use std::collections::BTreeMap;

pub struct Buffer {
  lines: BTreeMap<usize, String>,
}

impl Buffer {
  fn from_text(content: String) -> Self {
    Buffer {
      lines: BTreeMap::new(),
    }
  }
  fn from_file(file: String) -> Self {
    Buffer {
      lines: BTreeMap::new(),
    }
  }
}
