use super::tree::*;
use crate::inode_impl;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;

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

#[test]
fn new() {
  let terminal_size = size!(18, 10);
  let tree = Tree::new(terminal_size).unwrap();
  assert!(tree.is_empty());
  assert!(tree.len() == 1);
}

#[test]
fn raw_move_position_by1() {
  // test_log_init();

  let s1 = rect!(0, 0, 20, 20);
  let n1 = TestValue::new(1, s1);
  let nid1 = n1.id();

  let s2 = rect!(0, 0, 20, 20);
  let n2 = TestValue::new(2, s2);
  let nid2 = n2.id();

  let s3 = rect!(0, 0, 1, 1);
  let n3 = TestValue::new(3, s3);
  let nid3 = n3.id();

  /*
   * The tree looks like:
   * ```
   *           n1
   *         /
   *        n2
   *       /
   *      n3
   * ```
   */
  let mut tree = Itree::new();
  tree.new_root(n1);
  tree.new_with_parent(nid1, n2);
  tree.new_with_parent(nid2, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  print_node!(n1, "n1");
  print_node!(n2, "n2");
  print_node!(n3, "n3");

  // n3 Move: (x, y)
  let moves: Vec<(isize, isize)> = vec![
    (-10, -4),
    (2, -7),
    (1, 90),
    (-70, 41),
    (23, -4),
    (49, -121),
    (8, 3),
    (-10, -7),
    (6, 8),
  ];
  let expects: Vec<IRect> = vec![
    rect!(-10, -4, -9, -3),
    rect!(-8, -11, -7, -10),
    rect!(-7, 79, -6, 80),
    rect!(-77, 120, -76, 121),
    rect!(-54, 116, -53, 117),
    rect!(-5, -5, -4, -4),
    rect!(3, -2, 4, -1),
    rect!(-7, -9, -6, -8),
    rect!(-1, -1, 0, 0),
  ];

  for (i, m) in moves.iter().enumerate() {
    let x = m.0;
    let y = m.1;
    tree.move_by(nid3, x, y);
    let actual = *tree.node(nid3).unwrap().shape();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert!(actual == expect);
  }
}

#[test]
fn raw_move_position_to1() {
  test_log_init();

  let s1 = rect!(0, 0, 20, 20);
  let n1 = TestValue::new(1, s1);
  let nid1 = n1.id();

  let s2 = rect!(0, 0, 20, 20);
  let n2 = TestValue::new(2, s2);
  let nid2 = n2.id();

  let s3 = rect!(0, 0, 1, 1);
  let n3 = TestValue::new(3, s3);
  let nid3 = n3.id();

  /*
   * The tree looks like:
   * ```
   *           n1
   *         /
   *        n2
   *       /
   *      n3
   * ```
   */
  let mut tree = Itree::new();
  tree.new_root(n1);
  tree.new_with_parent(nid1, n2);
  tree.new_with_parent(nid2, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  print_node!(n1, "n1");
  print_node!(n2, "n2");
  print_node!(n3, "n3");

  // n3 Move: (x, y)
  let moves: Vec<(isize, isize)> = vec![
    (-10, -4),
    (2, -7),
    (1, 90),
    (-70, 41),
    (23, -4),
    (49, -121),
    (8, 3),
    (-10, -7),
    (6, 8),
  ];
  let expects: Vec<IRect> = vec![
    rect!(-10, -4, -9, -3),
    rect!(2, -7, 3, -6),
    rect!(1, 90, 2, 91),
    rect!(-70, 41, -69, 42),
    rect!(23, -4, 24, -3),
    rect!(49, -121, 50, -120),
    rect!(8, 3, 9, 4),
    rect!(-10, -7, -9, -6),
    rect!(6, 8, 7, 9),
  ];

  for (i, m) in moves.iter().enumerate() {
    let x = m.0;
    let y = m.1;
    tree.move_to(nid3, x, y);
    let actual = *tree.node(nid3).unwrap().shape();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert!(actual == expect);
  }
}

#[test]
fn bounded_move_by1() {
  test_log_init();

  let s1 = rect!(0, 0, 20, 20);
  let n1 = TestValue::new(1, s1);
  let nid1 = n1.id();

  let s2 = rect!(0, 0, 20, 20);
  let n2 = TestValue::new(2, s2);
  let nid2 = n2.id();

  let s3 = rect!(0, 0, 1, 1);
  let n3 = TestValue::new(3, s3);
  let nid3 = n3.id();

  /*
   * The tree looks like:
   * ```
   *           n1
   *         /
   *        n2
   *       /
   *      n3
   * ```
   */
  let mut tree = Itree::new();
  tree.new_root(n1);
  tree.new_with_parent(nid1, n2);
  tree.new_with_parent(nid2, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  print_node!(n1, "n1");
  print_node!(n2, "n2");
  print_node!(n3, "n3");

  // n3 bounded move by: (x, y)
  let moves: Vec<(isize, isize)> = vec![
    (-10, -4),
    (2, -7),
    (1, 90),
    (-70, 41),
    (23, -4),
    (49, -121),
    (8, 3),
    (-10, -7),
    (6, 8),
  ];
  let expects: Vec<IRect> = vec![
    rect!(0, 0, 1, 1),
    rect!(2, 0, 3, 1),
    rect!(3, 19, 4, 20),
    rect!(0, 19, 1, 20),
    rect!(19, 15, 20, 16),
    rect!(19, 0, 20, 1),
    rect!(19, 3, 20, 4),
    rect!(9, 0, 10, 1),
    rect!(15, 8, 16, 9),
  ];

  for (i, m) in moves.iter().enumerate() {
    let x = m.0;
    let y = m.1;
    tree.bounded_move_by(nid3, x, y);
    let actual = *tree.node(nid3).unwrap().shape();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert!(actual == expect);
  }
}

#[test]
fn bounded_move_to1() {
  test_log_init();

  let s1 = rect!(0, 0, 20, 20);
  let n1 = TestValue::new(1, s1);
  let nid1 = n1.id();

  let s2 = rect!(0, 0, 20, 20);
  let n2 = TestValue::new(2, s2);
  let nid2 = n2.id();

  let s3 = rect!(0, 0, 1, 1);
  let n3 = TestValue::new(3, s3);
  let nid3 = n3.id();

  /*
   * The tree looks like:
   * ```
   *           n1
   *         /
   *        n2
   *       /
   *      n3
   * ```
   */
  let mut tree = Itree::new();
  tree.new_root(n1);
  tree.new_with_parent(nid1, n2);
  tree.new_with_parent(nid2, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  print_node!(n1, "n1");
  print_node!(n2, "n2");
  print_node!(n3, "n3");

  // n3 bounded move to: (x, y)
  let moves: Vec<(isize, isize)> = vec![
    (-10, -4),
    (2, -7),
    (1, 90),
    (-70, 41),
    (23, -4),
    (49, -121),
    (8, 3),
    (5, 6),
    (6, 8),
  ];
  let expects: Vec<IRect> = vec![
    rect!(0, 0, 1, 1),
    rect!(2, 0, 3, 1),
    rect!(1, 19, 2, 20),
    rect!(0, 19, 1, 20),
    rect!(19, 0, 20, 1),
    rect!(19, 0, 20, 1),
    rect!(8, 3, 9, 4),
    rect!(5, 6, 6, 7),
    rect!(6, 8, 7, 9),
  ];

  for (i, m) in moves.iter().enumerate() {
    let x = m.0;
    let y = m.1;
    tree.reserved_move_position_to(nid3, x, y);
    let actual = *tree.node(nid3).unwrap().shape();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert!(actual == expect);
  }
}
