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
}

impl TestValue {
  pub fn new(id: TreeNodeId, ctx: TreeContextWk) -> Self {
    Self {
      __node: InodeBase::new(id, ctx),
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
      height: taffy::prelude::length(10_u16),
      width: taffy::prelude::length(10_u16),
    },
    ..Default::default()
  };

  let nid1 = tree
    .context()
    .borrow_mut()
    .new_leaf_default(style.clone(), "n1")
    .unwrap();
  let n1 = TestValue::new(nid1, Rc::downgrade(&tree.context()));
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

  let style1 = Style {
    size: taffy::Size {
      height: taffy::prelude::length(20_u16),
      width: taffy::prelude::length(20_u16),
    },
    ..Default::default()
  };
  let nid1 = tree
    .context()
    .borrow_mut()
    .new_leaf_default(style1, "n1")
    .unwrap();
  let n1 = TestValue::new(nid1, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid1, n1);

  let style2 = Style {
    size: taffy::Size {
      height: taffy::prelude::percent(1.0),
      width: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };
  let nid2 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid1, style2, "n2")
    .unwrap();
  let n2 = TestValue::new(nid2, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid2, n2);

  let style3 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::prelude::length(0_u16),
      top: taffy::prelude::length(0_u16),
      right: taffy::prelude::auto(),
      bottom: taffy::prelude::auto(),
    },
    size: taffy::Size {
      height: taffy::prelude::length(1_u16),
      width: taffy::prelude::length(1_u16),
    },
    ..Default::default()
  };
  let nid3 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid2, style3, "n3")
    .unwrap();
  let n3 = TestValue::new(nid3, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid3, n3);

  tree.context().borrow_mut().compute_layout(nid1).unwrap();

  // n3 Move: (x, y)
  let moves: Vec<(isize, isize)> = vec![(-10, -4), (2, -7), (1, 90), (-70, 41)];
  let expects: Vec<IRect> = vec![
    rect!(-10, -4, -9, -3),
    rect!(2, -7, 3, -6),
    rect!(1, 90, 2, 91),
    rect!(-70, 41, -69, 42),
  ];

  for (i, m) in moves.iter().enumerate() {
    let x = m.0;
    let y = m.1;
    let ctx = tree.context();
    let ctx = ctx.borrow();
    let actual = tree.raw_move_position_by(&ctx, nid3, x, y).unwrap();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert_eq!(actual, expect);
  }
}

#[test]
fn raw_move_position_to1() {
  test_log_init();

  let mut tree = Itree::new();
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
  let style1 = Style {
    size: taffy::Size {
      height: taffy::prelude::length(20_u16),
      width: taffy::prelude::length(20_u16),
    },
    ..Default::default()
  };
  let nid1 = tree
    .context()
    .borrow_mut()
    .new_leaf_default(style1, "n1")
    .unwrap();
  let n1 = TestValue::new(nid1, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid1, n1);

  let style2 = Style {
    size: taffy::Size {
      height: taffy::prelude::percent(1.0),
      width: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };
  let nid2 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid1, style2, "n2")
    .unwrap();
  let n2 = TestValue::new(nid2, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid2, n2);

  let style3 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::prelude::length(0_u16),
      top: taffy::prelude::length(0_u16),
      right: taffy::prelude::auto(),
      bottom: taffy::prelude::auto(),
    },
    size: taffy::Size {
      height: taffy::prelude::length(1_u16),
      width: taffy::prelude::length(1_u16),
    },
    ..Default::default()
  };
  let nid3 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid2, style3, "n3")
    .unwrap();
  let n3 = TestValue::new(nid3, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid3, n3);

  tree.context().borrow_mut().compute_layout(nid1).unwrap();

  // n3 Move: (x, y)
  let moves: Vec<(isize, isize)> = vec![(-10, -4), (2, -7), (1, 90), (-70, 41)];
  let expects: Vec<IRect> = vec![
    rect!(-10, -4, -9, -3),
    rect!(2, -7, 3, -6),
    rect!(1, 90, 2, 91),
    rect!(-70, 41, -69, 42),
  ];

  for (i, m) in moves.iter().enumerate() {
    let x = m.0;
    let y = m.1;
    let ctx = tree.context();
    let ctx = ctx.borrow();
    let actual = tree.raw_move_position_to(&ctx, nid3, x, y).unwrap();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert_eq!(actual, expect);
  }
}

#[test]
fn reserved_move_position_by1() {
  test_log_init();

  let mut tree = Itree::new();
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
  let style1 = Style {
    size: taffy::Size {
      height: taffy::prelude::length(20_u16),
      width: taffy::prelude::length(20_u16),
    },
    ..Default::default()
  };
  let nid1 = tree
    .context()
    .borrow_mut()
    .new_leaf_default(style1, "n1")
    .unwrap();
  let n1 = TestValue::new(nid1, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid1, n1);

  let style2 = Style {
    size: taffy::Size {
      height: taffy::prelude::percent(1.0),
      width: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };
  let nid2 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid1, style2, "n2")
    .unwrap();
  let n2 = TestValue::new(nid2, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid2, n2);

  let style3 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::prelude::length(0_u16),
      top: taffy::prelude::length(0_u16),
      right: taffy::prelude::auto(),
      bottom: taffy::prelude::auto(),
    },
    size: taffy::Size {
      height: taffy::prelude::length(1_u16),
      width: taffy::prelude::length(1_u16),
    },
    ..Default::default()
  };
  let nid3 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid2, style3, "n3")
    .unwrap();
  let n3 = TestValue::new(nid3, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid3, n3);

  tree.context().borrow_mut().compute_layout(nid1).unwrap();

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
    rect!(1, 19, 2, 20),
    rect!(0, 19, 1, 20),
    rect!(19, 0, 20, 1),
    rect!(19, 0, 20, 1),
    rect!(8, 3, 9, 4),
    rect!(0, 0, 1, 1),
    rect!(6, 8, 7, 9),
  ];

  for (i, m) in moves.iter().enumerate() {
    let x = m.0;
    let y = m.1;
    let ctx = tree.context();
    let ctx = ctx.borrow();
    let actual = tree
      .move_position_by(&ctx, nid3, x, y, TruncatePolicy::RESERVED)
      .unwrap();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert_eq!(actual, expect);
  }
}

#[test]
fn reserved_move_position_to1() {
  test_log_init();

  let mut tree = Itree::new();
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
  let style1 = Style {
    size: taffy::Size {
      height: taffy::prelude::length(20_u16),
      width: taffy::prelude::length(20_u16),
    },
    ..Default::default()
  };
  let nid1 = tree
    .context()
    .borrow_mut()
    .new_leaf_default(style1, "n1")
    .unwrap();
  let n1 = TestValue::new(nid1, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid1, n1);

  let style2 = Style {
    size: taffy::Size {
      height: taffy::prelude::percent(1.0),
      width: taffy::prelude::percent(1.0),
    },
    ..Default::default()
  };
  let nid2 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid1, style2, "n2")
    .unwrap();
  let n2 = TestValue::new(nid2, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid2, n2);

  let style3 = Style {
    position: taffy::Position::Absolute,
    inset: taffy::Rect {
      left: taffy::prelude::length(0_u16),
      top: taffy::prelude::length(0_u16),
      right: taffy::prelude::auto(),
      bottom: taffy::prelude::auto(),
    },
    size: taffy::Size {
      height: taffy::prelude::length(1_u16),
      width: taffy::prelude::length(1_u16),
    },
    ..Default::default()
  };
  let nid3 = tree
    .context()
    .borrow_mut()
    .new_with_parent_default(nid2, style3, "n3")
    .unwrap();
  let n3 = TestValue::new(nid3, Rc::downgrade(&tree.context()));
  tree.nodes_mut().insert(nid3, n3);

  tree.context().borrow_mut().compute_layout(nid1).unwrap();

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
    let ctx = tree.context();
    let ctx = ctx.borrow();
    let actual = tree
      .move_position_to(&ctx, nid3, x, y, TruncatePolicy::RESERVED)
      .unwrap();
    let expect = expects[i];
    info!("i:{:?}, actual:{:?}, expect:{:?}", i, actual, expect);
    assert_eq!(actual, expect);
  }
}
