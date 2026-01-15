use super::range_tree_map::*;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;

#[test]
fn test1() {
  test_log_init();

  let mut tree: RangeTreeMap<usize, i32> = RangeTreeMap::new();
  tree.insert(10..20, 1);
  info!("tree:{:?}", tree);

  for i in 0..10 {
    assert_eq!(tree.query(i), None);
  }
  for i in 10..20 {
    assert_eq!(tree.query(i), Some(&1));
  }
  for i in 20..50 {
    assert_eq!(tree.query(i), None);
  }
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
