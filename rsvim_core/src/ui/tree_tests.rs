use super::tree::*;
use crate::inode_impl;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;

#[derive(Clone, Debug)]
struct TestValue {
  pub __node: InodeBase,
  pub value: i32,
}

inode_impl!(TestValue);

impl TestValue {
  pub fn new(id: TreeNodeId, ctx: TreeContextWk, value: i32) -> Self {
    TestValue {
      __node: InodeBase::new(id, ctx),
      value,
    }
  }
}

#[test]
fn new() {
  let terminal_size = size!(18, 10);
  let tree = Tree::new(terminal_size).unwrap();
  assert!(tree.is_empty());
  assert!(tree.len() == 1);
}
