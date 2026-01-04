#![allow(unused_imports, dead_code, unused_variables)]

use super::inode::*;
use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
// use crate::tests::log::init as test_log_init;
use taffy::Style;

// Test node
#[derive(Clone, Debug)]
struct TestNode {
  pub __node: InodeBase,
  pub value: usize,
}

inode_impl!(TestNode);

impl TestNode {
  pub fn new(id: TreeNodeId, ctx: TreeContextWk, value: usize) -> Self {
    TestNode {
      __node: InodeBase::new(id, ctx),
      value,
    }
  }
}

#[test]
fn new() {
  // test_log_init();

  let mut ctx = TreeContext::new();
  let style = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(1.0),
      height: taffy::Dimension::from_length(1.0),
    },
    ..Default::default()
  };
  let nid1 = ctx.new_leaf_default(style.clone(), "n1").unwrap();
  let nid2 = ctx.new_leaf_default(style.clone(), "n2").unwrap();
  ctx.compute_layout().unwrap();

  let ctx = TreeContext::to_rc(ctx);

  let n1 = TestNode::new(nid1, Rc::downgrade(&ctx), 1);
  let n2 = TestNode::new(nid1, Rc::downgrade(&ctx), 2);

  assert!(n1.id() < n2.id());
  assert_eq!(n1.id(), nid1);
  assert_eq!(n2.id(), nid2);
  assert!(n1.enabled());
  assert!(n2.enabled());
}
