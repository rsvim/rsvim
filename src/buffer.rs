//! The VIM buffer.

use ropey::Rope;
use std::io::{Read, Result as IoResult, Write};

pub struct Buffer {
  rope: Rope,
}

impl Buffer {
  fn from_str(text: &str) -> Self {
    Buffer {
      rope: Rope::from_str(text),
    }
  }

  fn from_reader<T>(reader: T) -> IoResult<Self>
  where
    T: Read,
  {
    let res = Rope::from_reader(reader);
    match res {
      Ok(rope) => Ok(Buffer { rope }),
      Err(e) => Err(e),
    }
  }

  fn write_to<T>(&self, writer: T) -> IoResult<()>
  where
    T: Write,
  {
    self.rope.write_to(writer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use tempfile::tempfile;

  #[test]
  fn should_be_equal_on_read_and_write() {
    let buf1 = Buffer::from_str("Hello");
    let tmp1 = tempfile().unwrap();
    buf1.write_to(tmp1).unwrap();
  }
}
