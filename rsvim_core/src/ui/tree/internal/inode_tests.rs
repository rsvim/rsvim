use super::inode::*;

use crate::inode_impl;
use crate::prelude::*;
// use crate::tests::log::init as test_log_init;

use std::cell::RefCell;

// Test node
#[derive(Copy, Clone, Debug)]
struct TestNode {
  pub value: usize,
  pub base: InodeBase,
}

impl TestNode {
  pub fn new(value: usize, shape: IRect) -> Self {
    TestNode {
      value,
      base: InodeBase::new(shape),
    }
  }
}

inode_impl!(TestNode, base);

#[test]
fn new() {
  // test_log_init();

  let n1 = TestNode::new(1, IRect::new((0, 0), (0, 0)));
  let n2 = TestNode::new(2, IRect::new((1, 2), (3, 4)));
  let n1 = RefCell::new(n1);
  let n2 = RefCell::new(n2);

  assert!(n1.borrow().id() < n2.borrow().id());
  assert_eq!(n1.borrow().value, 1);
  assert_eq!(n2.borrow().value, 2);

  n1.borrow_mut().value = 3;
  n2.borrow_mut().value = 4;
  assert_eq!(n1.borrow().value, 3);
  assert_eq!(n2.borrow().value, 4);

  assert_eq!(n1.borrow().depth(), 0);
  assert_eq!(n1.borrow().zindex(), 0);
  assert!(n1.borrow().enabled());
  assert!(n1.borrow().visible());

  assert_eq!(*n1.borrow().shape(), IRect::new((0, 0), (0, 0)));
  assert_eq!(*n2.borrow().shape(), IRect::new((1, 2), (3, 4)));
}

#[test]
fn next_node_id1() {
  assert!(next_node_id() > 0);
}
