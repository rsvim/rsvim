use super::inode::*;
use crate::inode_impl;
use crate::ui::tree::TaffyTreeWk;
use crate::ui::tree::new_layout_tree;
// use crate::tests::log::init as test_log_init;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::Style;

// Test node
#[derive(Clone, Debug)]
struct TestNode {
  pub base: InodeBase,
  pub value: usize,
}

impl TestNode {
  pub fn new(layout_tree: TaffyTreeWk, style: Style, value: usize) -> Self {
    TestNode {
      base: InodeBase::new(layout_tree, style),
      value,
    }
  }
}

inode_impl!(TestNode, base);

#[test]
fn new() {
  // test_log_init();

  let layout_tree = new_layout_tree();
  let n1 = TestNode::new(
    Rc::downgrade(&layout_tree),
    Style {
      ..Default::default()
    },
    1,
  );
  let n2 = TestNode::new(
    Rc::downgrade(&layout_tree),
    Style {
      ..Default::default()
    },
    2,
  );
  let n1 = RefCell::new(n1);
  let n2 = RefCell::new(n2);

  assert!(n1.borrow().id() < n2.borrow().id());
  assert_eq!(n1.borrow().value, 1);
  assert_eq!(n2.borrow().value, 2);

  n1.borrow_mut().value = 3;
  n2.borrow_mut().value = 4;
  assert_eq!(n1.borrow().value, 3);
  assert_eq!(n2.borrow().value, 4);
}

#[test]
fn next_node_id1() {
  assert!(next_node_id() > 0);
}
