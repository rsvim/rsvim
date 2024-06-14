//! The VIM buffer.

use ropey::Rope;

pub struct Buffer {
  pub rope: Rope,
}

impl Buffer {
  fn new(rope: Rope) -> Self {
    Buffer { rope }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use tempfile::tempfile;

  #[test]
  fn should_be_equal_on_read_and_write() {
    let rop1 = Rope::from_str("Hello");
    let buf1 = Buffer::new(rop1);
    let tmp1 = tempfile().unwrap();
    buf1.rope.write_to(tmp1).unwrap();

    let rop2 = Rope::from_reader(File::open("Cargo.toml").unwrap()).unwrap();
    let buf2 = Buffer::new(rop2);
    let tmp2 = tempfile().unwrap();
    buf2.rope.write_to(tmp2).unwrap();
  }
}
