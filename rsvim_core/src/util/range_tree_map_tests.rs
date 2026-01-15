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
  tree.insert(15..25, 2);
  println!("插入 key(start:15, end:25) = value(2)");
  // tree.print_all();
  println!();

  // 重置并测试用例 2
  println!("=== 测试用例 2 ===");
  let mut tree2 = RangeTreeMap::new();
  tree2.insert(10..20, 1);
  tree2.insert(15..25, 2);
  tree2.insert(11..13, 3);
  println!("依次插入:");
  println!("  key(start:10, end:20) = value(1)");
  println!("  key(start:15, end:25) = value(2)");
  println!("  key(start:11, end:13) = value(3)");
  // tree2.print_all();
}
