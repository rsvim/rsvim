use super::context::*;
use crate::inode_impl;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::InodeBase;
use crate::ui::tree::internal::Inodeable;
use itertools::Itertools;
use taffy::Style;
use taffy::prelude::FromLength;
use taffy::prelude::TaffyAuto;

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

macro_rules! print_node {
  ($node: ident, $name: expr) => {
    info!("{}: {:?}", $name, $node.clone());
  };
}

macro_rules! assert_node_actual_shape_eq {
  ($node: ident, $expect: expr, $index: expr) => {
    assert_eq!(*$node.actual_shape(), $expect, "index:{:?}", $index,);
  };
}

macro_rules! assert_node_value_eq {
  ($node: ident, $expect: expr) => {
    assert_eq!($node.value, $expect);
  };
}

#[test]
fn new() {
  // test_log_init();

  let mut ctx = TreeContext::new();

  let nid1 = ctx
    .new_leaf_default(
      Style {
        size: taffy::Size {
          width: taffy::Dimension::from_length(10_u16),
          height: taffy::Dimension::from_length(10_u16),
        },
        ..Default::default()
      },
      "n1",
    )
    .unwrap();

  ctx.compute_layout().unwrap();

  assert_eq!(ctx.len(), 1);
  assert_eq!(ctx.root(), nid1);
  assert_eq!(ctx.shape(nid1).copied().unwrap(), rect!(0, 0, 10, 10));
  assert_eq!(
    ctx.actual_shape(nid1).copied().unwrap(),
    rect!(0, 0, 10, 10)
  );
  assert!(ctx.parent(nid1).is_none());
  assert!(ctx.children(nid1).unwrap().is_empty());
}

#[test]
fn new_child1() {
  // test_log_init();

  let mut ctx = TreeContext::new();

  /*
   * The tree looks like:
   * ```
   *           n1
   *         /   \
   *        n2   n3
   *      /  \     \
   *     n4  n5    n6
   * ```
   */

  let style = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(1_u16),
      height: taffy::Dimension::from_length(1_u16),
    },
    ..Default::default()
  };
  let nid1 = ctx.new_leaf_default(style.clone(), "n1").unwrap();
  let nid2 = ctx
    .new_with_parent_default(nid1, style.clone(), "n2")
    .unwrap();
  let nid3 = ctx
    .new_with_parent_default(nid1, style.clone(), "n3")
    .unwrap();
  let nid4 = ctx
    .new_with_parent_default(nid2, style.clone(), "n4")
    .unwrap();
  let nid5 = ctx
    .new_with_parent_default(nid2, style.clone(), "n5")
    .unwrap();
  let nid6 = ctx
    .new_with_parent_default(nid3, style.clone(), "n6")
    .unwrap();
  ctx.compute_layout().unwrap();

  assert_eq!(ctx.root(), nid1);
  assert!(nid1 < nid2);
  assert!(nid2 < nid3);
  assert!(nid3 < nid4);
  assert!(nid4 < nid5);
  assert!(nid5 < nid6);

  assert_eq!(ctx.children(nid1).unwrap().len(), 2);
  assert_eq!(ctx.children(nid2).unwrap().len(), 2);
  assert_eq!(ctx.children(nid3).unwrap().len(), 1);
  assert_eq!(ctx.children(nid4).unwrap().len(), 0);
  assert_eq!(ctx.children(nid5).unwrap().len(), 0);
  assert_eq!(ctx.children(nid6).unwrap().len(), 0);

  let contains_child = |parent_id: TreeNodeId, child_id: TreeNodeId| -> bool {
    ctx
      .children(parent_id)
      .unwrap()
      .iter()
      .filter(|cid| **cid == child_id)
      .collect::<Vec<_>>()
      .len()
      == 1
  };

  assert!(contains_child(nid1, nid2));
  assert!(contains_child(nid1, nid3));
  assert!(!contains_child(nid1, nid4));
  assert!(!contains_child(nid1, nid5));
  assert!(!contains_child(nid1, nid6));

  assert!(contains_child(nid2, nid4));
  assert!(contains_child(nid2, nid5));
  assert!(!contains_child(nid2, nid6));

  assert!(contains_child(nid3, nid6));
  assert!(!contains_child(nid3, nid4));
  assert!(!contains_child(nid3, nid5));
}

#[test]
fn new_child2() {
  // test_log_init();

  let mut ctx = TreeContext::new();
  let style = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(1_u16),
      height: taffy::Dimension::from_length(1_u16),
    },
    ..Default::default()
  };

  /*
   * The tree looks like:
   * ```
   *           n1
   *         /   \
   *        n2   n3
   *      /  \     \
   *     n4  n5    n6
   *           \
   *            n7
   *           / \
   *         n8   n9
   * ```
   */

  let nid1 = ctx.new_leaf_default(style.clone(), "n1").unwrap();
  let nid2 = ctx
    .new_with_parent_default(nid1, style.clone(), "n2")
    .unwrap();
  let nid3 = ctx
    .new_with_parent_default(nid1, style.clone(), "n3")
    .unwrap();
  let nid4 = ctx
    .new_with_parent_default(nid2, style.clone(), "n4")
    .unwrap();
  let nid5 = ctx
    .new_with_parent_default(nid2, style.clone(), "n5")
    .unwrap();
  let nid6 = ctx
    .new_with_parent_default(nid3, style.clone(), "n6")
    .unwrap();
  let nid7 = ctx
    .new_with_parent_default(nid5, style.clone(), "n7")
    .unwrap();
  let nid8 = ctx
    .new_with_parent_default(nid7, style.clone(), "n8")
    .unwrap();
  let nid9 = ctx
    .new_with_parent_default(nid7, style.clone(), "n9")
    .unwrap();

  assert_eq!(ctx.root(), nid1);

  assert!(nid1 < nid2);
  assert!(nid2 < nid3);
  assert!(nid3 < nid4);
  assert!(nid4 < nid5);
  assert!(nid5 < nid6);
  assert!(nid6 < nid7);
  assert!(nid7 < nid8);
  assert!(nid8 < nid9);

  assert_eq!(ctx.children(nid1).unwrap().len(), 2);
  assert_eq!(ctx.children(nid2).unwrap().len(), 2);
  assert_eq!(ctx.children(nid3).unwrap().len(), 1);
  assert_eq!(ctx.children(nid4).unwrap().len(), 0);
  assert_eq!(ctx.children(nid5).unwrap().len(), 1);
  assert_eq!(ctx.children(nid6).unwrap().len(), 0);
  assert_eq!(ctx.children(nid7).unwrap().len(), 2);
  assert_eq!(ctx.children(nid8).unwrap().len(), 0);
  assert_eq!(ctx.children(nid9).unwrap().len(), 0);

  let contains_child = |parent_id: TreeNodeId, child_id: TreeNodeId| -> bool {
    let result = ctx
      .children(parent_id)
      .unwrap()
      .iter()
      .filter(|cid| **cid == child_id)
      .collect::<Vec<_>>()
      .len()
      == 1;
    info!(
      "parent: {:?}, child: {:?}, children_ids: {:?}, contains: {:?}",
      parent_id,
      child_id,
      ctx.children(parent_id),
      result
    );
    result
  };

  assert!(contains_child(nid1, nid2));
  assert!(contains_child(nid1, nid3));
  assert!(!contains_child(nid1, nid4));
  assert!(!contains_child(nid1, nid5));
  assert!(!contains_child(nid1, nid7));

  assert!(contains_child(nid2, nid4));
  assert!(contains_child(nid2, nid5));
  assert!(!contains_child(nid2, nid7));

  assert!(contains_child(nid3, nid6));
  assert!(!contains_child(nid3, nid7));
  assert!(!contains_child(nid3, nid4));
  assert!(!contains_child(nid3, nid5));

  assert!(contains_child(nid5, nid7));
  assert!(contains_child(nid7, nid8));
  assert!(contains_child(nid7, nid9));
}

#[test]
fn shape1() {
  // test_log_init();

  let mut ctx = TreeContext::new();

  /*
   * The tree looks like:
   * ```
   *           n1
   *         /   \
   *        n2   n3
   *      /  \     \
   *     n4  n5    n6
   *           \
   *            n7
   *           / \
   *         n8   n9
   * ```
   */

  let style1 = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(20_u16),
      height: taffy::Dimension::from_length(20_u16),
    },
    ..Default::default()
  };
  let nid1 = ctx.new_leaf_default(style1, "n1").unwrap();
  let s1 = rect!(0, 0, 20, 20);
  let us1 = rect!(0, 0, 20, 20);

  let style2 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(0_u16),
      top: taffy::LengthPercentageAuto::from_length(0_u16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(15_u16),
      height: taffy::Dimension::from_length(15_u16),
    },
    ..Default::default()
  };
  let nid2 = ctx.new_with_parent_default(nid1, style2, "n2").unwrap();
  let s2 = rect!(0, 0, 15, 15);
  let us2 = rect!(0, 0, 15, 15);

  let style3 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(10_u16),
      top: taffy::LengthPercentageAuto::from_length(10_u16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(8_u16),
      height: taffy::Dimension::from_length(9_u16),
    },
    ..Default::default()
  };
  let nid3 = ctx.new_with_parent_default(nid1, style3, "n3").unwrap();
  let s3 = rect!(10, 10, 18, 19);
  let us3 = rect!(10, 10, 18, 19);

  let style4 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(3_u16),
      top: taffy::LengthPercentageAuto::from_length(5_u16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(12_u16),
      height: taffy::Dimension::from_length(9_u16),
    },
    ..Default::default()
  };
  let nid4 = ctx.new_with_parent_default(nid2, style4, "n4").unwrap();
  let s4 = rect!(3, 5, 20, 14);
  let us4 = rect!(3, 5, 15, 14);

  let style5 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(-3_i16),
      top: taffy::LengthPercentageAuto::from_length(-5_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(13_u16),
      height: taffy::Dimension::from_length(25_u16),
    },
    ..Default::default()
  };
  let nid5 = ctx.new_with_parent_default(nid2, style5, "n5").unwrap();
  let s5 = rect!(-3, -5, 10, 20);
  let us5 = rect!(0, 0, 10, 15);

  let style6 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(3_i16),
      top: taffy::LengthPercentageAuto::from_length(6_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(3_u16),
      height: taffy::Dimension::from_length(4_u16),
    },
    ..Default::default()
  };
  let nid6 = ctx.new_with_parent_default(nid3, style6, "n6").unwrap();
  let s6 = rect!(3, 6, 6, 10);
  let us6 = rect!(13, 16, 16, 19);

  let style7 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(3_i16),
      top: taffy::LengthPercentageAuto::from_length(6_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(12_u16),
      height: taffy::Dimension::from_length(19_u16),
    },
    ..Default::default()
  };
  let s7 = rect!(3, 6, 15, 25);
  let us7 = rect!(3, 6, 10, 15);
  let nid7 = ctx.new_with_parent_default(nid5, style7, "n7").unwrap();

  let style8 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(-1_i16),
      top: taffy::LengthPercentageAuto::from_length(-2_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(3_u16),
      height: taffy::Dimension::from_length(3_u16),
    },
    ..Default::default()
  };
  let nid8 = ctx.new_with_parent_default(nid7, style8, "n8").unwrap();
  let s8 = rect!(-1, -2, 2, 1);
  let us8 = rect!(3, 6, 5, 7);

  let style9 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(5_i16),
      top: taffy::LengthPercentageAuto::from_length(6_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(4_u16),
      height: taffy::Dimension::from_length(2_u16),
    },
    ..Default::default()
  };
  let s9 = rect!(5, 6, 9, 8);
  let us9 = rect!(8, 12, 10, 14);
  let nid9 = ctx.new_with_parent_default(nid7, style9, "n9").unwrap();

  ctx.compute_layout().unwrap();

  assert_eq!(ctx.root(), nid1);

  let nids = [nid1, nid2, nid3, nid4, nid5, nid6, nid7, nid8, nid9];
  let expect_actual_shapes = [us1, us2, us3, us4, us5, us6, us7, us8, us9];
  let expect_shapes = [s1, s2, s3, s4, s5, s6, s7, s8, s9];
  for (i, nid) in nids.iter().enumerate() {
    let expect_us = expect_actual_shapes[i];
    let expect_s = expect_shapes[i];
    let actual_us = ctx.actual_shape(*nid).copied().unwrap();
    let actual_s = ctx.shape(*nid).copied().unwrap();
    assert_eq!(expect_us, actual_us);
    assert_eq!(expect_s, actual_s);
  }
}

#[test]
fn shape2() {
  // test_log_init();

  let mut ctx = TreeContext::new();

  /*
   * The tree looks like:
   * ```
   *           n1
   *         /   \
   *        n2   n3
   *         \
   *         n4
   *        /
   *       n5
   *      /
   *     n6
   * ```
   */

  let style1 = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(20_u16),
      height: taffy::Dimension::from_length(20_u16),
    },
    ..Default::default()
  };
  let s1 = rect!(0, 0, 20, 20);
  let us1 = rect!(0, 0, 20, 20);
  let nid1 = ctx.new_leaf_default(style1, "n1").unwrap();

  let style2 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(0_i16),
      top: taffy::LengthPercentageAuto::from_length(0_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(20_u16),
      height: taffy::Dimension::from_length(20_u16),
    },
    ..Default::default()
  };
  let nid2 = ctx.new_with_parent_default(nid1, style2, "n2").unwrap();
  let s2 = rect!(0, 0, 20, 20);
  let us2 = rect!(0, 0, 20, 20);

  let style3 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(-2_i16),
      top: taffy::LengthPercentageAuto::from_length(-2_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(3_u16),
      height: taffy::Dimension::from_length(2_u16),
    },
    ..Default::default()
  };
  let nid3 = ctx.new_with_parent_default(nid1, style3, "n3").unwrap();
  let s3 = rect!(-2, -2, -1, 0);
  let us3 = rect!(0, 0, 0, 0);

  let style4 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(3_i16),
      top: taffy::LengthPercentageAuto::from_length(5_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(17_u16),
      height: taffy::Dimension::from_length(15_u16),
    },
    ..Default::default()
  };
  let nid4 = ctx.new_with_parent_default(nid2, style4, "n4").unwrap();
  let s4 = rect!(3, 5, 20, 20);
  let us4 = rect!(3, 5, 20, 20);

  let style5 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(-3_i16),
      top: taffy::LengthPercentageAuto::from_length(-5_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(18_u16),
      height: taffy::Dimension::from_length(25_u16),
    },
    ..Default::default()
  };
  let nid5 = ctx.new_with_parent_default(nid4, style5, "n5").unwrap();
  let s5 = rect!(-3, -5, 15, 20);
  let us5 = rect!(3, 5, 18, 20);

  let style6 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(8_i16),
      top: taffy::LengthPercentageAuto::from_length(13_i16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      width: taffy::Dimension::from_length(10_u16),
      height: taffy::Dimension::from_length(12_u16),
    },
    ..Default::default()
  };
  let nid6 = ctx.new_with_parent_default(nid5, style6, "n6").unwrap();
  let s6 = rect!(8, 13, 18, 25);
  let us6 = rect!(11, 18, 18, 20);

  ctx.compute_layout().unwrap();

  assert_eq!(ctx.root(), nid1);

  let nids = [nid1, nid2, nid3, nid4, nid5, nid6];
  let expect_actual_shapes = [us1, us2, us3, us4, us5, us6];
  let expect_shapes = [s1, s2, s3, s4, s5, s6];
  for (i, nid) in nids.iter().enumerate() {
    let expect_us = expect_actual_shapes[i];
    let expect_s = expect_shapes[i];
    let actual_us = ctx.actual_shape(*nid).copied().unwrap();
    let actual_s = ctx.shape(*nid).copied().unwrap();
    assert_eq!(expect_us, actual_us);
    assert_eq!(expect_s, actual_s);
  }
}

#[test]
fn children1() {
  // test_log_init();

  let mut ctx = TreeContext::new();

  /*
   * The tree looks like:
   * ```
   *             n1
   *         /        \
   *       n2, n3, n4, n5
   * ```
   */

  let style1 = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(10_u16),
      height: taffy::Dimension::from_length(10_u16),
    },
    ..Default::default()
  };

  let nid1 = ctx.new_leaf_default(style1.clone(), "n1").unwrap();
  let mut nids = vec!["n2", "n3", "n4", "n5"]
    .iter()
    .map(|name| {
      ctx
        .new_with_parent_default(nid1, style1.clone(), name)
        .unwrap()
    })
    .collect_vec();
  nids.insert(0, nid1);

  assert!(ctx.root() == nids[0]);
  assert_eq!(ctx.children(nids[0]).unwrap().len(), 4);
  for nid in nids.iter().skip(1) {
    assert!(ctx.children(*nid).unwrap().is_empty());
  }
}

fn make_tree(n: usize) -> (Vec<TreeNodeId>, TreeContext) {
  let style = Style {
    size: taffy::Size {
      width: taffy::Dimension::from_length(10_u16),
      height: taffy::Dimension::from_length(10_u16),
    },
    ..Default::default()
  };

  let mut ctx = TreeContext::new();
  let root_id = ctx.new_leaf_default(style.clone(), "root").unwrap();

  let mut nids: Vec<TreeNodeId> = vec![];
  nids.push(root_id);

  for _ in 1..n {
    let nid = ctx
      .new_with_parent_default(root_id, style.clone(), "node")
      .unwrap();
    nids.push(nid);
  }

  (nids, ctx)
}

#[test]
fn remove_child1() {
  // test_log_init();

  let (nids, mut ctx) = make_tree(5);
  let root_id = ctx.root();
  let remove2_id = ctx.remove_child(root_id, nids[2]).unwrap();
  let remove4_id = ctx.remove_child(root_id, nids[4]).unwrap();

  assert!(!ctx.children(root_id).unwrap().contains(&remove2_id));
  assert!(!ctx.children(root_id).unwrap().contains(&remove4_id));

  let remove1_id = ctx.remove_child(root_id, nids[1]).unwrap();
  let remove3_id = ctx.remove_child(root_id, nids[3]).unwrap();

  // 1,2,(3),4,(5)
  assert!(!ctx.children(root_id).unwrap().contains(&remove1_id));
  assert!(!ctx.children(root_id).unwrap().contains(&remove3_id));
}

// #[test]
// fn move_by1() {
//   // test_log_init();
//
//   let s1 = rect!(0, 0, 20, 20);
//   let n1 = TestValue::new(1, s1);
//   let nid1 = n1.id();
//
//   let s2 = rect!(0, 0, 20, 20);
//   let n2 = TestValue::new(2, s2);
//   let nid2 = n2.id();
//
//   let s3 = rect!(0, 0, 1, 1);
//   let n3 = TestValue::new(3, s3);
//   let nid3 = n3.id();
//
//   /*
//    * The tree looks like:
//    * ```
//    *           n1
//    *         /
//    *        n2
//    *       /
//    *      n3
//    * ```
//    */
//   let mut tree = Itree::new();
//   tree.new_root(n1);
//   tree.new_with_parent(nid1, n2);
//   tree.new_with_parent(nid2, n3);
//
//   let n1 = tree.node(nid1).unwrap();
//   let n2 = tree.node(nid2).unwrap();
//   let n3 = tree.node(nid3).unwrap();
//   print_node!(n1, "n1");
//   print_node!(n2, "n2");
//   print_node!(n3, "n3");
//
//   // n3 Move: (x, y)
//   let moves: Vec<(isize, isize)> = vec![
//     (-10, -4),
//     (2, -7),
//     (1, 90),
//     (-70, 41),
//     (23, -4),
//     (49, -121),
//     (8, 3),
//     (-10, -7),
//     (6, 8),
//   ];
//   let expects: Vec<IRect> = vec![
//     rect!(-10, -4, -9, -3),
//     rect!(-8, -11, -7, -10),
//     rect!(-7, 79, -6, 80),
//     rect!(-77, 120, -76, 121),
//     rect!(-54, 116, -53, 117),
//     rect!(-5, -5, -4, -4),
//     rect!(3, -2, 4, -1),
//     rect!(-7, -9, -6, -8),
//     rect!(-1, -1, 0, 0),
//   ];
//
//   for (i, m) in moves.iter().enumerate() {
//     let x = m.0;
//     let y = m.1;
//     tree.move_by(nid3, x, y);
//     let actual = *tree.node(nid3).unwrap().shape();
//     let expect = expects[i];
//     info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
//     assert!(actual == expect);
//   }
// }
//
// #[test]
// fn bounded_move_by1() {
//   test_log_init();
//
//   let s1 = rect!(0, 0, 20, 20);
//   let n1 = TestValue::new(1, s1);
//   let nid1 = n1.id();
//
//   let s2 = rect!(0, 0, 20, 20);
//   let n2 = TestValue::new(2, s2);
//   let nid2 = n2.id();
//
//   let s3 = rect!(0, 0, 1, 1);
//   let n3 = TestValue::new(3, s3);
//   let nid3 = n3.id();
//
//   /*
//    * The tree looks like:
//    * ```
//    *           n1
//    *         /
//    *        n2
//    *       /
//    *      n3
//    * ```
//    */
//   let mut tree = Itree::new();
//   tree.new_root(n1);
//   tree.new_with_parent(nid1, n2);
//   tree.new_with_parent(nid2, n3);
//
//   let n1 = tree.node(nid1).unwrap();
//   let n2 = tree.node(nid2).unwrap();
//   let n3 = tree.node(nid3).unwrap();
//   print_node!(n1, "n1");
//   print_node!(n2, "n2");
//   print_node!(n3, "n3");
//
//   // n3 bounded move by: (x, y)
//   let moves: Vec<(isize, isize)> = vec![
//     (-10, -4),
//     (2, -7),
//     (1, 90),
//     (-70, 41),
//     (23, -4),
//     (49, -121),
//     (8, 3),
//     (-10, -7),
//     (6, 8),
//   ];
//   let expects: Vec<IRect> = vec![
//     rect!(0, 0, 1, 1),
//     rect!(2, 0, 3, 1),
//     rect!(3, 19, 4, 20),
//     rect!(0, 19, 1, 20),
//     rect!(19, 15, 20, 16),
//     rect!(19, 0, 20, 1),
//     rect!(19, 3, 20, 4),
//     rect!(9, 0, 10, 1),
//     rect!(15, 8, 16, 9),
//   ];
//
//   for (i, m) in moves.iter().enumerate() {
//     let x = m.0;
//     let y = m.1;
//     tree.bounded_move_by(nid3, x, y);
//     let actual = *tree.node(nid3).unwrap().shape();
//     let expect = expects[i];
//     info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
//     assert!(actual == expect);
//   }
// }
//
// #[test]
// fn move_to1() {
//   test_log_init();
//
//   let s1 = rect!(0, 0, 20, 20);
//   let n1 = TestValue::new(1, s1);
//   let nid1 = n1.id();
//
//   let s2 = rect!(0, 0, 20, 20);
//   let n2 = TestValue::new(2, s2);
//   let nid2 = n2.id();
//
//   let s3 = rect!(0, 0, 1, 1);
//   let n3 = TestValue::new(3, s3);
//   let nid3 = n3.id();
//
//   /*
//    * The tree looks like:
//    * ```
//    *           n1
//    *         /
//    *        n2
//    *       /
//    *      n3
//    * ```
//    */
//   let mut tree = Itree::new();
//   tree.new_root(n1);
//   tree.new_with_parent(nid1, n2);
//   tree.new_with_parent(nid2, n3);
//
//   let n1 = tree.node(nid1).unwrap();
//   let n2 = tree.node(nid2).unwrap();
//   let n3 = tree.node(nid3).unwrap();
//   print_node!(n1, "n1");
//   print_node!(n2, "n2");
//   print_node!(n3, "n3");
//
//   // n3 Move: (x, y)
//   let moves: Vec<(isize, isize)> = vec![
//     (-10, -4),
//     (2, -7),
//     (1, 90),
//     (-70, 41),
//     (23, -4),
//     (49, -121),
//     (8, 3),
//     (-10, -7),
//     (6, 8),
//   ];
//   let expects: Vec<IRect> = vec![
//     rect!(-10, -4, -9, -3),
//     rect!(2, -7, 3, -6),
//     rect!(1, 90, 2, 91),
//     rect!(-70, 41, -69, 42),
//     rect!(23, -4, 24, -3),
//     rect!(49, -121, 50, -120),
//     rect!(8, 3, 9, 4),
//     rect!(-10, -7, -9, -6),
//     rect!(6, 8, 7, 9),
//   ];
//
//   for (i, m) in moves.iter().enumerate() {
//     let x = m.0;
//     let y = m.1;
//     tree.move_to(nid3, x, y);
//     let actual = *tree.node(nid3).unwrap().shape();
//     let expect = expects[i];
//     info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
//     assert!(actual == expect);
//   }
// }
//
// #[test]
// fn bounded_move_to1() {
//   test_log_init();
//
//   let s1 = rect!(0, 0, 20, 20);
//   let n1 = TestValue::new(1, s1);
//   let nid1 = n1.id();
//
//   let s2 = rect!(0, 0, 20, 20);
//   let n2 = TestValue::new(2, s2);
//   let nid2 = n2.id();
//
//   let s3 = rect!(0, 0, 1, 1);
//   let n3 = TestValue::new(3, s3);
//   let nid3 = n3.id();
//
//   /*
//    * The tree looks like:
//    * ```
//    *           n1
//    *         /
//    *        n2
//    *       /
//    *      n3
//    * ```
//    */
//   let mut tree = Itree::new();
//   tree.new_root(n1);
//   tree.new_with_parent(nid1, n2);
//   tree.new_with_parent(nid2, n3);
//
//   let n1 = tree.node(nid1).unwrap();
//   let n2 = tree.node(nid2).unwrap();
//   let n3 = tree.node(nid3).unwrap();
//   print_node!(n1, "n1");
//   print_node!(n2, "n2");
//   print_node!(n3, "n3");
//
//   // n3 bounded move to: (x, y)
//   let moves: Vec<(isize, isize)> = vec![
//     (-10, -4),
//     (2, -7),
//     (1, 90),
//     (-70, 41),
//     (23, -4),
//     (49, -121),
//     (8, 3),
//     (5, 6),
//     (6, 8),
//   ];
//   let expects: Vec<IRect> = vec![
//     rect!(0, 0, 1, 1),
//     rect!(2, 0, 3, 1),
//     rect!(1, 19, 2, 20),
//     rect!(0, 19, 1, 20),
//     rect!(19, 0, 20, 1),
//     rect!(19, 0, 20, 1),
//     rect!(8, 3, 9, 4),
//     rect!(5, 6, 6, 7),
//     rect!(6, 8, 7, 9),
//   ];
//
//   for (i, m) in moves.iter().enumerate() {
//     let x = m.0;
//     let y = m.1;
//     tree.reserved_move_position_to(nid3, x, y);
//     let actual = *tree.node(nid3).unwrap().shape();
//     let expect = expects[i];
//     info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
//     assert!(actual == expect);
//   }
// }
