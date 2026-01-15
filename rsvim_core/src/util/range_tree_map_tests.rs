use super::range_tree_map::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use std::ops::Range;

fn assert_hit(
  tree: &RangeTreeMap<usize, i32>,
  range: Range<usize>,
  value: i32,
) {
  for i in range {
    assert_eq!(tree.query(i), Some(&value));
  }
}

fn assert_miss(tree: &RangeTreeMap<usize, i32>, range: Range<usize>) {
  for i in range {
    assert_eq!(tree.query(i), None);
  }
}

#[test]
fn insert1() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  tree.insert(10..20, 1);
  info!("tree:{:?}", tree);
  assert_hit(&tree, 10..20, 1);

  tree.insert(10..20, 2);
  info!("tree:{:?}", tree);
  assert_hit(&tree, 10..20, 2);

  tree.insert(0..10, 3);
  info!("tree:{:?}", tree);
  assert_hit(&tree, 0..10, 3);
  assert_hit(&tree, 10..20, 2);

  tree.insert(20..30, 4);
  info!("tree:{:?}", tree);
  assert_hit(&tree, 0..10, 3);
  assert_hit(&tree, 10..20, 2);
  assert_hit(&tree, 20..30, 4);
}

#[test]
fn insert2() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  // [15----------25]
  tree.insert(15..25, 1);
  info!("tree-1:{:?}", tree);
  assert_hit(&tree, 15..25, 1);

  // {10----[15----20}-----25]
  tree.insert(10..20, 2);
  info!("tree-2:{:?}", tree);
  assert_hit(&tree, 10..20, 2);
  assert_hit(&tree, 20..25, 1);

  // {10----[15----20}-----25]
  tree.insert(15..25, 3);
  info!("tree-3:{:?}", tree);
  assert_hit(&tree, 10..15, 2);
  assert_hit(&tree, 15..25, 3);

  // {10-(11-13)--[15----20}-----25]
  tree.insert(11..13, 4);
  info!("tree-4:{:?}", tree);
  assert_hit(&tree, 10..11, 2);
  assert_hit(&tree, 11..13, 4);
  assert_hit(&tree, 13..15, 2);
  assert_hit(&tree, 15..25, 3);
}

#[test]
fn insert3() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  // [15----------25]
  tree.insert(15..25, 1);
  info!("tree-1:{:?}", tree);
  assert_hit(&tree, 15..25, 1);

  // {[15--------25]----------50}
  tree.insert(15..50, 2);
  info!("tree-2:{:?}", tree);
  assert_hit(&tree, 15..50, 2);

  // {[15--------25](25---30)----------50}
  tree.insert(25..30, 3);
  info!("tree-3:{:?}", tree);
  assert_hit(&tree, 15..25, 2);
  assert_hit(&tree, 25..30, 3);
  assert_hit(&tree, 30..50, 2);

  // {[15--------25](25-[27--30)----------50}----60]
  tree.insert(27..60, 4);
  info!("tree-4:{:?}", tree);
  assert_hit(&tree, 15..25, 2);
  assert_hit(&tree, 25..27, 3);
  assert_hit(&tree, 27..60, 4);
}

#[test]
fn insert4() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  // [15----------25]
  tree.insert(15..25, 1);
  info!("tree-1:{:?}", tree);
  assert_hit(&tree, 15..25, 1);

  // [15--------25]{25----------35}
  tree.insert(25..35, 2);
  info!("tree-2:{:?}", tree);
  assert_hit(&tree, 15..25, 1);
  assert_hit(&tree, 25..35, 2);

  // [15----(20--------30)------35}
  tree.insert(20..30, 3);
  info!("tree-3:{:?}", tree);
  assert_hit(&tree, 15..20, 1);
  assert_hit(&tree, 20..30, 3);
  assert_hit(&tree, 30..35, 2);

  // {10-----17}---(20----25]{25----30)------35}
  tree.insert(10..17, 4);
  info!("tree-4:{:?}", tree);
  assert_hit(&tree, 10..17, 4);
  assert_hit(&tree, 17..20, 1);
  assert_hit(&tree, 20..30, 3);
  assert_hit(&tree, 30..35, 2);

  // {10-----17}---(20----25]{25----30)---{33----40}
  tree.insert(33..40, 5);
  info!("tree-5:{:?}", tree);
  assert_hit(&tree, 10..17, 4);
  assert_hit(&tree, 17..20, 1);
  assert_hit(&tree, 20..30, 3);
  assert_hit(&tree, 30..33, 2);
  assert_hit(&tree, 33..40, 5);
}

#[test]
fn remove1() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  // [15----------25]
  tree.insert(15..25, 1);
  info!("tree-1:{:?}", tree);
  assert_hit(&tree, 15..25, 1);

  // [15--------25]{25----------35}
  tree.insert(25..35, 2);
  info!("tree-2:{:?}", tree);
  assert_hit(&tree, 15..25, 1);
  assert_hit(&tree, 25..35, 2);

  // [15----(20--------30)------35}
  tree.insert(20..30, 3);
  info!("tree-3:{:?}", tree);
  assert_hit(&tree, 15..20, 1);
  assert_hit(&tree, 20..30, 3);
  assert_hit(&tree, 30..35, 2);

  // [15----(20--22)   (28--30)------35}
  tree.remove(22..28);
  info!("tree-4:{:?}", tree);
  assert_hit(&tree, 15..20, 1);
  assert_hit(&tree, 20..22, 3);
  assert_miss(&tree, 22..28);
  assert_hit(&tree, 28..30, 3);
  assert_hit(&tree, 30..35, 2);

  // [15--17]   {33--35}
  tree.remove(17..33);
  info!("tree-5:{:?}", tree);
  assert_hit(&tree, 15..17, 1);
  assert_miss(&tree, 17..33);
  assert_hit(&tree, 33..35, 2);
}
