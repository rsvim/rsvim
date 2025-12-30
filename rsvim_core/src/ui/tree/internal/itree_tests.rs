use super::itree::*;
use crate::inode_impl;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use crate::ui::tree::*;
use taffy::Style;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;
use taffy::prelude::TaffyAuto;

#[derive(Clone, Debug)]
struct TestValue {
  pub __node: InodeBase,
  pub value: i32,
}

impl TestValue {
  pub fn new(id: TreeNodeId, ctx: TreeContextWk, value: i32) -> Self {
    TestValue {
      __node: InodeBase::new(id, ctx),
      value,
    }
  }
}

inode_impl!(TestValue);

#[test]
fn new() {
  // test_log_init();

  let mut tree = Itree::new();

  let style = Style {
    size: taffy::Size {
      height: taffy::Dimension::from_length(10_u16),
      width: taffy::Dimension::from_length(10_u16),
    },
    ..Default::default()
  };

  let nid1 = tree
    .context()
    .borrow_mut()
    .new_leaf_default(style.clone(), "n1")
    .unwrap();
  let n1 = TestValue::new(nid1, Rc::downgrade(&tree.context()), 1);
  tree.nodes_mut().insert(nid1, n1);

  assert_eq!(tree.len(), 1);
  assert_eq!(tree.root_id(), nid1);
  assert!(tree.parent_id(nid1).is_none());
  assert!(tree.children_ids(nid1).unwrap().is_empty());
}

#[test]
fn raw_move_position_by1() {
  // test_log_init();

  let mut tree = Itree::new();

  let style1 = Style {
    size: taffy::Size {
      height: taffy::Dimension::from_length(20_u16),
      width: taffy::Dimension::from_length(20_u16),
    },
    ..Default::default()
  };
  let nid1 = tree
    .context()
    .borrow_mut()
    .new_leaf_default(style1, "n1")
    .unwrap();
  let n1 = TestValue::new(nid1, Rc::downgrade(&tree.context()), 1);
  tree.nodes_mut().insert(nid1, n1);

  let style2 = Style {
    size: taffy::Size {
      height: taffy::Dimension::from_percent(1.0),
      width: taffy::Dimension::from_percent(1.0),
    },
    ..Default::default()
  };
  let nid2 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid1, style2, "n2")
    .unwrap();
  let n2 = TestValue::new(nid2, Rc::downgrade(&tree.context()), 2);
  tree.nodes_mut().insert(nid2, n2);

  let style3 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(0_u16),
      top: taffy::LengthPercentageAuto::from_length(0_u16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    },
    size: taffy::Size {
      height: taffy::Dimension::from_length(1_u16),
      width: taffy::Dimension::from_length(1_u16),
    },
    ..Default::default()
  };
  let nid3 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid2, style3, "n3")
    .unwrap();
  let n3 = TestValue::new(nid3, Rc::downgrade(&tree.context()), 3);
  tree.nodes_mut().insert(nid3, n3);

  tree.context().borrow_mut().compute_layout().unwrap();

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
  tree.insert_root(n1);
  tree.insert(nid1, n3);
  tree.insert(nid3, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid3).unwrap();
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
  tree.insert_root(n1);
  tree.insert(nid1, n2);
  tree.insert(nid2, n3);

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
fn reserved_move_position_by1() {
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
  tree.insert_root(n1);
  tree.insert(nid1, n2);
  tree.insert(nid2, n3);

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
fn reserved_move_position_to1() {
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
  tree.insert_root(n1);
  tree.insert(nid1, n2);
  tree.insert(nid2, n3);

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
    tree.bounded_move_to(nid3, x, y);
    let actual = *tree.node(nid3).unwrap().shape();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert!(actual == expect);
  }
}
