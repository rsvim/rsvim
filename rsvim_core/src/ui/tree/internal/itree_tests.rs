#![allow(unused_imports)]

use super::itree::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use crate::ui::tree::*;
use itertools::Itertools;
use taffy::prelude::FromPercent;
use taffy::prelude::TaffyAuto;
use taffy::prelude::TaffyMaxContent;

#[test]
fn new() {
  // test_log_init();

  let mut tree = Itree::new();
  let nid1 = tree
    .new_leaf(Style {
      ..Default::default()
    })
    .unwrap();

  assert_eq!(tree.len(), 1);
  assert_eq!(tree.parent(nid1), None);
  assert_eq!(tree.children(nid1), Ok(vec![]));
}

#[test]
fn insert1() {
  // test_log_init();

  let mut tree = Itree::new();
  let nid = (0..7)
    .map(|_i| {
      tree
        .new_leaf(Style {
          ..Default::default()
        })
        .unwrap()
    })
    .collect_vec();

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
  tree.add_child(nid[1], nid[2]).unwrap();
  tree.add_child(nid[1], nid[3]).unwrap();
  tree.add_child(nid[2], nid[4]).unwrap();
  tree.add_child(nid[2], nid[5]).unwrap();
  tree.add_child(nid[3], nid[6]).unwrap();

  assert!(tree.parent(nid[1]).is_none());
  assert!(nid[1] < nid[2]);
  assert!(nid[2] < nid[3]);
  assert!(nid[3] < nid[4]);
  assert!(nid[4] < nid[5]);
  assert!(nid[5] < nid[6]);

  assert_eq!(tree.children(nid[1]).unwrap().len(), 2);
  assert_eq!(tree.children(nid[2]).unwrap().len(), 2);
  assert_eq!(tree.children(nid[3]).unwrap().len(), 1);
  assert_eq!(tree.children(nid[4]).unwrap().len(), 0);
  assert_eq!(tree.children(nid[5]).unwrap().len(), 0);
  assert_eq!(tree.children(nid[6]).unwrap().len(), 0);

  let contains_child = |parent_id: TreeNodeId, child_id: TreeNodeId| -> bool {
    tree
      .children(parent_id)
      .unwrap()
      .iter()
      .filter(|cid| **cid == child_id)
      .collect::<Vec<_>>()
      .len()
      == 1
  };

  assert!(contains_child(nid[1], nid[2]));
  assert!(contains_child(nid[1], nid[3]));
  assert!(!contains_child(nid[1], nid[4]));
  assert!(!contains_child(nid[1], nid[5]));
  assert!(!contains_child(nid[1], nid[6]));

  assert!(contains_child(nid[2], nid[4]));
  assert!(contains_child(nid[2], nid[5]));
  assert!(!contains_child(nid[2], nid[6]));

  assert!(contains_child(nid[3], nid[6]));
  assert!(!contains_child(nid[3], nid[4]));
  assert!(!contains_child(nid[3], nid[5]));
}

#[test]
fn insert2() {
  // test_log_init();

  let mut tree = Itree::new();
  let nid = (0..10)
    .map(|_i| {
      tree
        .new_leaf(Style {
          ..Default::default()
        })
        .unwrap()
    })
    .collect_vec();

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
  tree.add_child(nid[1], nid[2]).unwrap();
  tree.add_child(nid[1], nid[3]).unwrap();
  tree.add_child(nid[2], nid[4]).unwrap();
  tree.add_child(nid[2], nid[5]).unwrap();
  tree.add_child(nid[3], nid[6]).unwrap();
  tree.add_child(nid[5], nid[7]).unwrap();
  tree.add_child(nid[7], nid[8]).unwrap();
  tree.add_child(nid[7], nid[9]).unwrap();

  assert!(tree.parent(nid[1]).is_none());
  assert!(nid[1] < nid[2]);
  assert!(nid[2] < nid[3]);
  assert!(nid[3] < nid[4]);
  assert!(nid[4] < nid[5]);
  assert!(nid[5] < nid[6]);
  assert!(nid[6] < nid[7]);
  assert!(nid[7] < nid[8]);
  assert!(nid[8] < nid[9]);

  assert_eq!(tree.children(nid[1]).unwrap().len(), 2);
  assert_eq!(tree.children(nid[2]).unwrap().len(), 2);
  assert_eq!(tree.children(nid[3]).unwrap().len(), 1);
  assert_eq!(tree.children(nid[4]).unwrap().len(), 0);
  assert_eq!(tree.children(nid[5]).unwrap().len(), 1);
  assert_eq!(tree.children(nid[6]).unwrap().len(), 0);
  assert_eq!(tree.children(nid[7]).unwrap().len(), 2);
  assert_eq!(tree.children(nid[8]).unwrap().len(), 0);
  assert_eq!(tree.children(nid[9]).unwrap().len(), 0);

  let contains_child = |parent_id: TreeNodeId, child_id: TreeNodeId| -> bool {
    tree
      .children(parent_id)
      .unwrap()
      .iter()
      .filter(|cid| **cid == child_id)
      .collect::<Vec<_>>()
      .len()
      == 1
  };

  assert!(contains_child(nid[1], nid[2]));
  assert!(contains_child(nid[1], nid[3]));
  assert!(!contains_child(nid[1], nid[4]));
  assert!(!contains_child(nid[1], nid[5]));
  assert!(!contains_child(nid[1], nid[7]));

  assert!(contains_child(nid[2], nid[4]));
  assert!(contains_child(nid[2], nid[5]));
  assert!(!contains_child(nid[2], nid[7]));

  assert!(contains_child(nid[3], nid[6]));
  assert!(!contains_child(nid[3], nid[7]));
  assert!(!contains_child(nid[3], nid[4]));
  assert!(!contains_child(nid[3], nid[5]));

  assert!(contains_child(nid[5], nid[7]));
  assert!(contains_child(nid[7], nid[8]));
  assert!(contains_child(nid[7], nid[9]));
}

#[test]
fn shape1() {
  test_log_init();

  let mut tree = Itree::new();
  let nid1 = tree
    .new_leaf(Style {
      size: taffy::Size {
        width: taffy::Dimension::from_length(20_u16),
        height: taffy::Dimension::from_length(20_u16),
      },
      ..Default::default()
    })
    .unwrap();
  let s1 = rect!(0, 0, 20, 20);
  let us1 = rect!(0, 0, 20, 20);

  let nid2 = tree
    .new_leaf(Style {
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
    })
    .unwrap();
  let s2 = rect!(0, 0, 15, 15);
  let us2 = rect!(0, 0, 15, 15);

  let nid3 = tree
    .new_leaf(Style {
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
    })
    .unwrap();
  let s3 = rect!(10, 10, 18, 19);
  let us3 = rect!(10, 10, 18, 19);

  let nid4 = tree
    .new_leaf(Style {
      position: taffy::Position::Absolute,
      inset: taffy::Rect {
        left: taffy::LengthPercentageAuto::from_length(3_u16),
        top: taffy::LengthPercentageAuto::from_length(5_u16),
        right: taffy::LengthPercentageAuto::AUTO,
        bottom: taffy::LengthPercentageAuto::AUTO,
      },
      size: taffy::Size {
        width: taffy::Dimension::from_length(17_u16),
        height: taffy::Dimension::from_length(9_u16),
      },
      ..Default::default()
    })
    .unwrap();
  let s4 = rect!(3, 5, 15, 14);
  let us4 = rect!(3, 5, 15, 14);

  let nid5 = tree
    .new_leaf(Style {
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
    })
    .unwrap();
  let s5 = rect!(0, 0, 10, 15);
  let us5 = rect!(0, 0, 10, 15);

  let nid6 = tree
    .new_leaf(Style {
      position: taffy::Position::Absolute,
      inset: taffy::Rect {
        left: taffy::LengthPercentageAuto::from_length(3_u16),
        top: taffy::LengthPercentageAuto::from_length(6_u16),
        right: taffy::LengthPercentageAuto::AUTO,
        bottom: taffy::LengthPercentageAuto::AUTO,
      },
      size: taffy::Size {
        width: taffy::Dimension::from_length(3_u16),
        height: taffy::Dimension::from_length(4_u16),
      },
      ..Default::default()
    })
    .unwrap();
  let s6 = rect!(3, 6, 6, 9);
  let us6 = rect!(13, 16, 16, 19);

  let nid7 = tree
    .new_leaf(Style {
      position: taffy::Position::Absolute,
      inset: taffy::Rect {
        left: taffy::LengthPercentageAuto::from_length(3_u16),
        top: taffy::LengthPercentageAuto::from_length(6_u16),
        right: taffy::LengthPercentageAuto::AUTO,
        bottom: taffy::LengthPercentageAuto::AUTO,
      },
      size: taffy::Size {
        width: taffy::Dimension::from_length(3_u16),
        height: taffy::Dimension::from_length(19_u16),
      },
      ..Default::default()
    })
    .unwrap();
  let s7 = rect!(3, 6, 6, 20);
  let us7 = rect!(3, 6, 10, 15);

  let nid8 = tree
    .new_leaf(Style {
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
    })
    .unwrap();
  let s8 = rect!(-1, -2, 2, 1);
  let us8 = rect!(3, 6, 5, 7);

  let nid9 = tree
    .new_leaf(Style {
      position: taffy::Position::Absolute,
      inset: taffy::Rect {
        left: taffy::LengthPercentageAuto::from_length(5_u16),
        top: taffy::LengthPercentageAuto::from_length(6_u16),
        right: taffy::LengthPercentageAuto::AUTO,
        bottom: taffy::LengthPercentageAuto::AUTO,
      },
      size: taffy::Size {
        width: taffy::Dimension::from_length(4_u16),
        height: taffy::Dimension::from_length(2_u16),
      },
      ..Default::default()
    })
    .unwrap();
  let s9 = rect!(5, 6, 9, 8);
  let us9 = rect!(8, 12, 10, 14);

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
  tree.add_child(nid1, nid2).unwrap();
  tree.add_child(nid1, nid3).unwrap();
  tree.add_child(nid2, nid4).unwrap();
  tree.add_child(nid2, nid5).unwrap();
  tree.add_child(nid3, nid6).unwrap();
  tree.add_child(nid5, nid7).unwrap();
  tree.add_child(nid7, nid8).unwrap();
  tree.add_child(nid7, nid9).unwrap();
  tree.compute_layout(nid1, taffy::Size::MAX_CONTENT).unwrap();

  assert!(tree.parent(nid1).is_none());

  let nids = [nid1, nid2, nid3, nid4, nid5, nid6, nid7, nid8, nid9];
  let expect_actual_shapes = [us1, us2, us3, us4, us5, us6, us7, us8, us9];
  let expect_shapes = [s1, s2, s3, s4, s5, s6, s7, s8, s9];
  for i in 0..9 {
    let expect_us = expect_actual_shapes[i];
    let expect_s = expect_shapes[i];
    let actual_us = tree.actual_shape(nids[i]).unwrap();
    let actual_s = tree.shape(nids[i]).unwrap();
    info!(
      "{},actual_shape:{:?}(expect)={:?}(actual),shape:{:?}(expect)={:?}(actual)",
      i + 1,
      expect_us,
      actual_us,
      expect_s,
      actual_s
    );
    assert_eq!(expect_us, actual_us);
    assert_eq!(expect_s, actual_s);
  }
}

#[test]
fn shape2() {
  // test_log_init();

  let mut tree = Itree::new();
  let nid1 = tree
    .new_leaf(Style {
      size: taffy::Size {
        width: taffy::Dimension::from_length(20_u16),
        height: taffy::Dimension::from_length(20_u16),
      },
      ..Default::default()
    })
    .unwrap();
  let s1 = rect!(0, 0, 20, 20);
  let us1 = rect!(0, 0, 20, 20);

  let nid2 = tree
    .new_leaf(Style {
      size: taffy::Size {
        width: taffy::Dimension::from_percent(0.99),
        height: taffy::Dimension::from_percent(0.99),
      },
      ..Default::default()
    })
    .unwrap();
  let s2 = rect!(0, 0, 20, 20);
  let us2 = rect!(0, 0, 20, 20);

  let nid3 = tree
    .new_leaf(Style {
      min_size: taffy::Size {
        width: taffy::Dimension::from_length(5_u16),
        height: taffy::Dimension::from_length(5_u16),
      },
      size: taffy::Size {
        width: taffy::Dimension::from_percent(0.01),
        height: taffy::Dimension::from_percent(0.01),
      },
      ..Default::default()
    })
    .unwrap();
  let s3 = rect!(-2, -2, -1, 0);
  let us3 = rect!(0, 0, 0, 0);

  let nid4 = tree
    .new_leaf(Style {
      min_size: taffy::Size {
        width: taffy::Dimension::from_length(5_u16),
        height: taffy::Dimension::from_length(5_u16),
      },
      size: taffy::Size {
        width: taffy::Dimension::from_percent(0.01),
        height: taffy::Dimension::from_percent(0.01),
      },
      ..Default::default()
    })
    .unwrap();
  let s4 = rect!(3, 5, 20, 20);
  let us4 = rect!(3, 5, 20, 20);

  let nid5 = tree
    .new_leaf(Style {
      position: taffy::Position::Absolute,
      inset: taffy::Rect {
        left: taffy::LengthPercentageAuto::from_length(-3_i16),
        top: taffy::LengthPercentageAuto::from_length(-5_i16),
        right: taffy::LengthPercentageAuto::AUTO,
        bottom: taffy::LengthPercentageAuto::AUTO,
      },
      min_size: taffy::Size {
        width: taffy::Dimension::from_length(18_u16),
        height: taffy::Dimension::from_length(25_u16),
      },
      ..Default::default()
    })
    .unwrap();
  let s5 = rect!(-3, -5, 15, 20);
  let us5 = rect!(3, 5, 18, 20);

  let nid6 = tree
    .new_leaf(Style {
      position: taffy::Position::Absolute,
      inset: taffy::Rect {
        left: taffy::LengthPercentageAuto::from_length(8_i16),
        top: taffy::LengthPercentageAuto::from_length(13_i16),
        right: taffy::LengthPercentageAuto::AUTO,
        bottom: taffy::LengthPercentageAuto::AUTO,
      },
      min_size: taffy::Size {
        width: taffy::Dimension::from_length(10_u16),
        height: taffy::Dimension::from_length(12_u16),
      },
      ..Default::default()
    })
    .unwrap();
  let s6 = rect!(8, 13, 18, 25);
  let us6 = rect!(11, 18, 18, 20);

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
  tree.add_child(nid1, nid2).unwrap();
  tree.add_child(nid1, nid3).unwrap();
  tree.add_child(nid2, nid4).unwrap();
  tree.add_child(nid4, nid5).unwrap();
  tree.add_child(nid5, nid6).unwrap();
  tree.compute_layout(nid1, taffy::Size::MAX_CONTENT).unwrap();

  assert!(tree.parent(nid1).is_none());

  let nids = [nid1, nid2, nid3, nid4, nid5, nid6];
  let expect_actual_shapes: [U16Rect; 6] = [us1, us2, us3, us4, us5, us6];
  let expect_shapes: [IRect; 6] = [s1, s2, s3, s4, s5, s6];
  for i in 0..6 {
    let expect_us = expect_actual_shapes[i];
    let expect_s = expect_shapes[i];
    let actual_us = tree.actual_shape(nids[i]).unwrap();
    let actual_s = tree.shape(nids[i]).unwrap();
    assert_eq!(expect_s, actual_s);
    assert_eq!(expect_us, actual_us);
  }
}
