use super::range_tree_map::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use std::ops::Range;

fn assert_range(
  tree: &RangeTreeMap<usize, i32>,
  range: Range<usize>,
  value: i32,
) {
  for i in range.start.saturating_sub(100)..range.end.saturating_add(100) {
    if i >= range.start && i < range.end {
      assert_eq!(tree.query(i), Some(&value));
    } else {
      assert_ne!(tree.query(i), Some(&value));
    }
  }
}

#[test]
fn test1() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  tree.insert(10..20, 1);
  info!("tree:{:?}", tree);
  assert_range(&tree, 10..20, 1);

  tree.insert(10..20, 2);
  info!("tree:{:?}", tree);
  assert_range(&tree, 10..20, 2);

  tree.insert(0..10, 3);
  info!("tree:{:?}", tree);
  assert_range(&tree, 0..10, 3);
  assert_range(&tree, 10..20, 2);

  tree.insert(20..30, 4);
  info!("tree:{:?}", tree);
  assert_range(&tree, 0..10, 3);
  assert_range(&tree, 10..20, 2);
  assert_range(&tree, 20..30, 4);
}

#[test]
fn test2() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  // [15----------25]
  tree.insert(15..25, 1);
  info!("tree-1:{:?}", tree);
  assert_range(&tree, 15..25, 1);

  // {10----[15----20}-----25]
  tree.insert(10..20, 2);
  info!("tree-2:{:?}", tree);
  assert_range(&tree, 10..20, 2);
  assert_range(&tree, 20..25, 1);

  // {10----[15----20}-----25]
  tree.insert(15..25, 3);
  info!("tree-3:{:?}", tree);
  assert_range(&tree, 10..15, 2);
  assert_range(&tree, 15..25, 3);

  // {10-(11-13)--[15----20}-----25]
  tree.insert(11..13, 4);
  info!("tree-4:{:?}", tree);
  assert_range(&tree, 10..11, 2);
  assert_range(&tree, 11..13, 4);
  assert_range(&tree, 13..15, 2);
  assert_range(&tree, 15..25, 3);
}

#[test]
fn test3() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  // [15----------25]
  tree.insert(15..25, 1);
  info!("tree-1:{:?}", tree);
  assert_range(&tree, 15..25, 1);

  // {[15--------25]----------50}
  tree.insert(15..50, 2);
  info!("tree-2:{:?}", tree);
  assert_range(&tree, 15..50, 2);

  // {[15--------25](25---30)----------50}
  tree.insert(25..30, 3);
  info!("tree-3:{:?}", tree);
  assert_range(&tree, 15..25, 2);
  assert_range(&tree, 25..30, 3);
  assert_range(&tree, 30..50, 2);

  // {[15--------25](25-[27--30)----------50}----60]
  tree.insert(27..60, 4);
  info!("tree-4:{:?}", tree);
  assert_range(&tree, 15..25, 2);
  assert_range(&tree, 25..27, 3);
  assert_range(&tree, 27..60, 4);
}
