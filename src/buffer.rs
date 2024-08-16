//! The VIM buffer.

#![allow(dead_code)]

use std::convert::From;

use ropey::{Rope, RopeBuilder};

use crate::uuid;

pub type BufferId = usize;

#[derive(Clone, Debug)]
pub struct Buffer {
  id: BufferId,
  rope: Rope,
}

impl Buffer {
  pub fn new() -> Self {
    Buffer {
      id: uuid::next(),
      rope: Rope::new(),
    }
  }

  pub fn id(&self) -> BufferId {
    self.id
  }

  pub fn rope(&self) -> &Rope {
    &self.rope
  }

  pub fn rope_mut(&mut self) -> &mut Rope {
    &mut self.rope
  }
}

impl Default for Buffer {
  fn default() -> Self {
    Buffer::new()
  }
}

impl From<Rope> for Buffer {
  fn from(rope: Rope) -> Self {
    Buffer {
      id: uuid::next(),
      rope,
    }
  }
}

impl From<RopeBuilder> for Buffer {
  fn from(builder: RopeBuilder) -> Self {
    Buffer {
      id: uuid::next(),
      rope: builder.finish(),
    }
  }
}

impl PartialEq for Buffer {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}

impl Eq for Buffer {}

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs::File;
  use tempfile::tempfile;

  #[test]
  fn buffer_from1() {
    let rop1 = Rope::from_str("Hello");
    let buf1: Buffer = rop1.into();
    let tmp1 = tempfile().unwrap();
    buf1.rope.write_to(tmp1).unwrap();

    let rop2 = Rope::from_reader(File::open("Cargo.toml").unwrap()).unwrap();
    let buf2 = Buffer::from(rop2);
    let tmp2 = tempfile().unwrap();
    buf2.rope.write_to(tmp2).unwrap();
  }

  #[test]
  fn buffer_from2() {
    let mut builder1 = RopeBuilder::new();
    builder1.append("Hello");
    builder1.append("World");
    let buf1: Buffer = builder1.into();
    let tmp1 = tempfile().unwrap();
    buf1.rope.write_to(tmp1).unwrap();
  }
}
