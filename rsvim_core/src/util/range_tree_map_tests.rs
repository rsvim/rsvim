use super::range_tree_map::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use std::ops::Range;

fn assert_range(
  tree: &RangeTreeMap<usize, i32>,
  range: Range<usize>,
  value: i32,
) {
  for i in range.start.saturating_sub(100)..range.start {
    assert_eq!(tree.query(i), None);
  }
  for i in range.start..range.end {
    assert_eq!(tree.query(i), Some(&value));
  }
  for i in range.end..range.end.saturating_add(100) {
    assert_eq!(tree.query(i), None);
  }
}

#[test]
fn test1() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  tree.insert(10..20, 1);
  info!("tree:{:?}", tree);

  assert_range(&tree, 10..20, 1);
}

#[test]
fn test2() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  tree.insert(15..25, 1);
  info!("tree-1:{:?}", tree);

  tree.insert(10..20, 2);
  info!("tree-2:{:?}", tree);
  tree.insert(15..25, 3);
  info!("tree-3:{:?}", tree);
  tree.insert(11..13, 4);
  info!("tree-4:{:?}", tree);
}
