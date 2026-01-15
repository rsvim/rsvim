use super::range_tree_map::*;

#[test]
fn test1() {
  let mut tree: RangeMap<usize, i32> = RangeMap::new();

  // 测试用例 1
  println!("=== 测试用例 1 ===");
  tree.insert(10..20, 1);
  println!("插入 key(start:10, end:20) = value(1)");
  // tree.print_all();
  println!();

  tree.insert(15..25, 2);
  println!("插入 key(start:15, end:25) = value(2)");
  // tree.print_all();
  println!();

  // 重置并测试用例 2
  println!("=== 测试用例 2 ===");
  let mut tree2 = RangeMap::new();
  tree2.insert(10..20, 1);
  tree2.insert(15..25, 2);
  tree2.insert(11..13, 3);
  println!("依次插入:");
  println!("  key(start:10, end:20) = value(1)");
  println!("  key(start:15, end:25) = value(2)");
  println!("  key(start:11, end:13) = value(3)");
  // tree2.print_all();
}
