use super::tree::*;
use crate::prelude::*;

#[test]
fn new() {
  let terminal_size = U16Size::new(18, 10);
  let tree = Tree::new(terminal_size);
  assert!(tree.is_empty());
  assert!(tree.len() == 1);
}
