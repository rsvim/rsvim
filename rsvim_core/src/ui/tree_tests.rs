use super::tree::*;
use crate::prelude::*;
use crate::size;

#[test]
fn new() {
  let terminal_size = size!(18, 10);
  let tree = Tree::new(terminal_size);
  assert!(tree.is_empty());
  assert!(tree.len() == 1);
}
