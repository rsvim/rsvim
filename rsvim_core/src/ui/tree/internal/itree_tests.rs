use super::itree::*;
use crate::inode_impl;
use crate::prelude::*;
use crate::tests::log::init as test_log_init;
use crate::ui::tree::*;
use itertools::Itertools;

#[derive(Clone, Debug)]
struct TestValue {
  base: IrelationshipRc,
  id: TreeNodeId,
  value: i32,
}

inode_impl!(TestValue);

impl TestValue {
  pub fn new(base: IrelationshipRc, id: TreeNodeId, value: i32) -> Self {
    TestValue { base, id, value }
  }
}

macro_rules! assert_parent_child_depth {
  ($parent: ident, $child: ident) => {
    assert_eq!($parent.depth() + 1, $child.depth());
  };
}

macro_rules! assert_node_actual_shape_eq {
  ($node: ident, $expect: expr, $index: expr) => {
    assert_eq!(*$node.actual_shape(), $expect, "index:{:?}", $index,);
  };
}

macro_rules! assert_node_value_eq {
  ($node: ident, $expect: expr) => {
    assert_eq!($node.value, $expect);
  };
}

#[test]
fn new() {
  // test_log_init();

  let mut tree = Irelationship::new();
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

  let mut tree = Irelationship::new();
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

  assert!(contains_child(nid[2], nid[2]));
  assert!(contains_child(nid[2], nid[5]));
  assert!(!contains_child(nid[2], nid[6]));

  assert!(contains_child(nid[3], nid[6]));
  assert!(!contains_child(nid[3], nid[4]));
  assert!(!contains_child(nid[3], nid[5]));
}

#[test]
fn insert2() {
  // test_log_init();

  let mut tree = Irelationship::new();
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
  tree.add_child(nid[1], nid[2]);
  tree.add_child(nid[1], nid[3]);
  tree.add_child(nid[2], nid[4]);
  tree.add_child(nid[2], nid[5]);
  tree.add_child(nid[3], nid[6]);
  tree.add_child(nid[5], nid[7]);
  tree.add_child(nid[7], nid[8]);
  tree.add_child(nid[7], nid[9]);

  assert!(tree.parent(nid1).is_none());
  assert!(nid[1] < nid[2]);
  assert!(nid[2] < nid[3]);
  assert!(nid[3] < nid[4]);
  assert!(nid[4] < nid[5]);
  assert!(nid[5] < nid[6]);
  assert!(nid[6] < nid[7]);
  assert!(nid[7] < nid[8]);
  assert!(nid[8] < nid[9]);

  assert_eq!(tree.children_ids(nid1).len(), 2);
  assert_eq!(tree.children_ids(nid2).len(), 2);
  assert_eq!(tree.children_ids(nid3).len(), 1);
  assert_eq!(tree.children_ids(nid4).len(), 0);
  assert_eq!(tree.children_ids(nid5).len(), 1);
  assert_eq!(tree.children_ids(nid6).len(), 0);
  assert_eq!(tree.children_ids(nid7).len(), 2);
  assert_eq!(tree.children_ids(nid8).len(), 0);
  assert_eq!(tree.children_ids(nid9).len(), 0);

  let contains_child = |parent_id: TreeNodeId, child_id: TreeNodeId| -> bool {
    let result = tree
      .children_ids(parent_id)
      .iter()
      .filter(|cid| **cid == child_id)
      .collect::<Vec<_>>()
      .len()
      == 1;
    info!(
      "parent: {:?}, child: {:?}, children_ids: {:?}, contains: {:?}",
      parent_id,
      child_id,
      tree.children_ids(parent_id),
      result
    );
    result
  };

  assert!(contains_child(nid1, nid2));
  assert!(contains_child(nid1, nid3));
  assert!(!contains_child(nid1, nid4));
  assert!(!contains_child(nid1, nid5));
  assert!(!contains_child(nid1, nid7));

  assert!(contains_child(nid2, nid4));
  assert!(contains_child(nid2, nid5));
  assert!(!contains_child(nid2, nid7));

  assert!(contains_child(nid3, nid6));
  assert!(!contains_child(nid3, nid7));
  assert!(!contains_child(nid3, nid4));
  assert!(!contains_child(nid3, nid5));

  assert!(contains_child(nid5, nid7));
  assert!(contains_child(nid7, nid8));
  assert!(contains_child(nid7, nid9));
}

#[test]
fn shape1() {
  // test_log_init();

  let s1 = rect!(0, 0, 20, 20);
  let us1 = rect!(0, 0, 20, 20);
  let n1 = TestValue::new(1, s1);
  let nid1 = n1.id();

  let s2 = rect!(0, 0, 15, 15);
  let us2 = rect!(0, 0, 15, 15);
  let n2 = TestValue::new(2, s2);
  let nid2 = n2.id();

  let s3 = rect!(10, 10, 18, 19);
  let us3 = rect!(10, 10, 18, 19);
  let n3 = TestValue::new(3, s3);
  let nid3 = n3.id();

  let s4 = rect!(3, 5, 20, 14);
  let us4 = rect!(3, 5, 15, 14);
  let n4 = TestValue::new(4, s4);
  let nid4 = n4.id();

  let s5 = rect!(-3, -5, 10, 20);
  let us5 = rect!(0, 0, 10, 15);
  let n5 = TestValue::new(5, s5);
  let nid5 = n5.id();

  let s6 = rect!(3, 6, 6, 10);
  let us6 = rect!(13, 16, 16, 19);
  let n6 = TestValue::new(6, s6);
  let nid6 = n6.id();

  let s7 = rect!(3, 6, 15, 25);
  let us7 = rect!(3, 6, 10, 15);
  let n7 = TestValue::new(7, s7);
  let nid7 = n7.id();

  let s8 = rect!(-1, -2, 2, 1);
  let us8 = rect!(3, 6, 5, 7);
  let n8 = TestValue::new(8, s8);
  let nid8 = n8.id();

  let s9 = rect!(5, 6, 9, 8);
  let us9 = rect!(8, 12, 10, 14);
  let n9 = TestValue::new(9, s9);
  let nid9 = n9.id();

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
  let mut tree = Itree::new(n1);
  tree.insert(nid1, n2);
  tree.insert(nid1, n3);
  tree.insert(nid2, n4);
  tree.insert(nid2, n5);
  tree.insert(nid3, n6);
  tree.insert(nid5, n7);
  tree.insert(nid7, n8);
  tree.insert(nid7, n9);

  assert!(tree.root_id() == nid1);
  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  let n4 = tree.node(nid4).unwrap();
  let n5 = tree.node(nid5).unwrap();
  let n6 = tree.node(nid6).unwrap();
  let n7 = tree.node(nid7).unwrap();
  let n8 = tree.node(nid8).unwrap();
  let n9 = tree.node(nid9).unwrap();
  info!("n1:{:?}", n1);
  info!("n2:{:?}", n2);
  info!("n3:{:?}", n3);
  info!("n4:{:?}", n4);
  info!("n5:{:?}", n5);
  info!("n6:{:?}", n6);
  info!("n7:{:?}", n7);
  info!("n8:{:?}", n8);
  info!("n9:{:?}", n9);

  let expects = [us1, us2, us3, us4, us5, us6, us7, us8, us9];
  let nodes = [n1, n2, n3, n4, n5, n6, n7, n8, n9];
  for i in 0..9 {
    let expect = expects[i];
    let node = nodes[i];
    assert_node_actual_shape_eq!(node, expect, i);
  }
}

#[test]
fn shape2() {
  // test_log_init();

  let s1 = rect!(0, 0, 20, 20);
  let us1 = rect!(0, 0, 20, 20);
  let n1 = TestValue::new(1, s1);
  let nid1 = n1.id();

  let s2 = rect!(0, 0, 20, 20);
  let us2 = rect!(0, 0, 20, 20);
  let n2 = TestValue::new(2, s2);
  let nid2 = n2.id();

  let s3 = rect!(-2, -2, -1, 0);
  let us3 = rect!(0, 0, 0, 0);
  let n3 = TestValue::new(3, s3);
  let nid3 = n3.id();

  let s4 = rect!(3, 5, 20, 20);
  let us4 = rect!(3, 5, 20, 20);
  let n4 = TestValue::new(4, s4);
  let nid4 = n4.id();

  let s5 = rect!(-3, -5, 15, 20);
  let us5 = rect!(3, 5, 18, 20);
  let n5 = TestValue::new(5, s5);
  let nid5 = n5.id();

  let s6 = rect!(8, 13, 18, 25);
  let us6 = rect!(11, 18, 18, 20);
  let n6 = TestValue::new(6, s6);
  let nid6 = n6.id();

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
  let mut tree = Itree::new(n1);
  tree.insert(nid1, n2);
  tree.insert(nid1, n3);
  tree.insert(nid2, n4);
  tree.insert(nid4, n5);
  tree.insert(nid5, n6);

  assert!(tree.root_id() == nid1);
  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  let n4 = tree.node(nid4).unwrap();
  let n5 = tree.node(nid5).unwrap();
  let n6 = tree.node(nid6).unwrap();
  info!("n1:{:?}", n1);
  info!("n2:{:?}", n2);
  info!("n3:{:?}", n3);
  info!("n4:{:?}", n4);
  info!("n5:{:?}", n5);
  info!("n6:{:?}", n6);

  let expects = [us1, us2, us3, us4, us5, us6];
  let nodes = [n1, n2, n3, n4, n5, n6];
  for i in 0..6 {
    let expect = expects[i];
    let node = &nodes[i];
    assert_node_actual_shape_eq!(node, expect, i);
  }
}

#[test]
fn push1() {
  // test_log_init();

  let shape = rect!(0, 0, 10, 10);
  let node_values: Vec<i32> = [1, 2, 3, 4, 5].to_vec();
  let nodes: Vec<TestValue> = node_values
    .iter()
    .map(|value| TestValue::new(*value, shape))
    .collect();
  let nodes_ids: Vec<TreeNodeId> = nodes.iter().map(|n| n.id()).collect();

  /*
   * The tree looks like:
   * ```
   *             n1
   *         /        \
   *       n2, n3, n4, n5
   * ```
   */
  let mut tree = Itree::new(nodes[0]);
  for node in nodes.iter().skip(1) {
    tree.insert(nodes_ids[0], *node);
  }

  assert!(tree.root_id() == nodes_ids[0]);
  assert!(tree.children_ids(nodes_ids[0]).len() == 4);
  assert!(!tree.children_ids(nodes_ids[0]).is_empty());
  for nid in nodes_ids.iter().skip(1) {
    assert!(tree.children_ids(*nid).is_empty());
  }

  for (i, nid) in nodes_ids.iter().enumerate() {
    let node = tree.node(*nid).unwrap();
    let expect = node_values[i];
    assert_node_value_eq!(node, expect);
  }

  let first1 = tree.children_ids(nodes_ids[0]).first().cloned();
  assert!(first1.is_some());
  assert_eq!(first1.unwrap(), nodes_ids[1]);

  let last1 = tree.children_ids(nodes_ids[0]).last().cloned();
  assert!(last1.is_some());
  assert_eq!(last1.unwrap(), nodes_ids[4]);

  for nid in nodes_ids.iter().skip(1) {
    let first = tree.children_ids(*nid).first().cloned();
    let last = tree.children_ids(*nid).last().cloned();
    assert!(first.is_none());
    assert!(last.is_none());
  }
}

fn make_tree(n: usize) -> (Vec<TreeNodeId>, Itree<TestValue>) {
  let mut value = 1;
  let mut node_ids: Vec<TreeNodeId> = vec![];

  let s = rect!(0, 0, 10, 10);
  let root = TestValue::new(value, s);
  let root_id = root.id();
  node_ids.push(root_id);
  value += 1;

  let mut tree = Itree::new(root);
  for _ in 1..n {
    let node = TestValue::new(value, s);
    let node_id = node.id();
    value += 1;
    tree.insert(root_id, node);
    node_ids.push(node_id);
  }

  (node_ids, tree)
}

#[test]
fn remove1() {
  // test_log_init();

  let (node_ids, mut tree) = make_tree(5);
  let remove2 = tree.remove(node_ids[2]);
  let remove4 = tree.remove(node_ids[4]);

  assert!(remove2.is_some());
  let remove2 = &remove2.unwrap();
  assert_node_value_eq!(remove2, 3);
  assert!(!tree.children_ids(tree.root_id()).contains(&remove2.id()));
  assert!(remove4.is_some());
  let remove4 = &remove4.unwrap();
  assert_node_value_eq!(remove4, 5);
  assert!(!tree.children_ids(tree.root_id()).contains(&remove4.id()));

  let remove1 = tree.remove(node_ids[1]);
  let remove3 = tree.remove(node_ids[3]);

  // 1,2,(3),4,(5)
  assert!(remove1.is_some());
  let remove1 = &remove1.unwrap();
  assert_node_value_eq!(remove1, 2);
  assert!(!tree.children_ids(tree.root_id()).contains(&remove1.id()));
  assert!(remove3.is_some());
  let remove3 = &remove3.unwrap();
  assert_node_value_eq!(remove3, 4);
  assert!(!tree.children_ids(tree.root_id()).contains(&remove3.id()));
}

#[test]
#[should_panic]
fn remove2() {
  // test_log_init();

  let (node_ids, mut tree) = make_tree(5);
  tree.remove(node_ids[0]);
}

#[test]
fn get1() {
  // test_log_init();

  let s1 = rect!(0, 0, 20, 20);
  let n1 = TestValue::new(1, s1);
  let nid1 = n1.id();

  let s2 = rect!(0, 0, 15, 15);
  let n2 = TestValue::new(2, s2);
  let nid2 = n2.id();

  let s3 = rect!(10, 10, 18, 19);
  let n3 = TestValue::new(3, s3);
  let nid3 = n3.id();

  let s4 = rect!(3, 5, 20, 14);
  let n4 = TestValue::new(4, s4);
  let nid4 = n4.id();

  let s5 = rect!(-3, -5, 10, 20);
  let n5 = TestValue::new(5, s5);
  let nid5 = n5.id();

  let s6 = rect!(3, 6, 6, 10);
  let n6 = TestValue::new(6, s6);
  let nid6 = n6.id();

  let s7 = rect!(3, 6, 15, 25);
  let n7 = TestValue::new(7, s7);
  let nid7 = n7.id();

  let s8 = rect!(-1, -2, 2, 1);
  let n8 = TestValue::new(8, s8);
  let nid8 = n8.id();

  let s9 = rect!(5, 6, 9, 8);
  let n9 = TestValue::new(9, s9);
  let nid9 = n9.id();

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
  let mut tree = Itree::new(n1);
  tree.insert(nid1, n2);
  tree.insert(nid1, n3);
  tree.insert(nid2, n4);
  tree.insert(nid2, n5);
  tree.insert(nid3, n6);
  tree.insert(nid5, n7);
  tree.insert(nid7, n8);
  tree.insert(nid7, n9);

  assert!(nid1 == tree.root_id());
  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  let n4 = tree.node(nid4).unwrap();
  let n5 = tree.node(nid5).unwrap();
  let n6 = tree.node(nid6).unwrap();
  let n7 = tree.node(nid7).unwrap();
  let n8 = tree.node(nid8).unwrap();
  let n9 = tree.node(nid9).unwrap();
  info!("n1:{:?}", n1);
  info!("n2:{:?}", n2);
  info!("n3:{:?}", n3);
  info!("n4:{:?}", n4);
  info!("n5:{:?}", n5);
  info!("n6:{:?}", n6);
  info!("n7:{:?}", n7);
  info!("n8:{:?}", n8);
  info!("n9:{:?}", n9);
}

#[test]
fn get2() {
  // test_log_init();

  let s1 = rect!(0, 0, 20, 20);
  let us1 = rect!(0, 0, 20, 20);
  let n1 = TestValue::new(1, s1);
  let nid1 = n1.id();

  let s2 = rect!(0, 0, 20, 20);
  let us2 = rect!(0, 0, 20, 20);
  let n2 = TestValue::new(2, s2);
  let nid2 = n2.id();

  let s3 = rect!(-2, -2, -1, 0);
  let us3 = rect!(0, 0, 0, 0);
  let n3 = TestValue::new(3, s3);
  let nid3 = n3.id();

  let s4 = rect!(3, 5, 20, 20);
  let us4 = rect!(3, 5, 20, 20);
  let n4 = TestValue::new(4, s4);
  let nid4 = n4.id();

  let s5 = rect!(-3, -5, 15, 20);
  let us5 = rect!(3, 5, 18, 20);
  let n5 = TestValue::new(5, s5);
  let nid5 = n5.id();

  let s6 = rect!(8, 13, 18, 25);
  let us6 = rect!(11, 18, 18, 20);
  let n6 = TestValue::new(6, s6);
  let nid6 = n6.id();

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
  let mut tree = Itree::new(n1);
  tree.insert(nid1, n2);
  tree.insert(nid1, n3);
  tree.insert(nid2, n4);
  tree.insert(nid4, n5);
  tree.insert(nid5, n6);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  let n4 = tree.node(nid4).unwrap();
  let n5 = tree.node(nid5).unwrap();
  let n6 = tree.node(nid6).unwrap();
  info!("n1:{:?}", n1);
  info!("n2:{:?}", n2);
  info!("n3:{:?}", n3);
  info!("n4:{:?}", n4);
  info!("n5:{:?}", n5);
  info!("n6:{:?}", n6);

  let expects = [us1, us2, us3, us4, us5, us6];
  let nodes = [n1, n2, n3, n4, n5, n6];
  for i in 0..6 {
    let expect = expects[i];
    let node = &nodes[i];
    assert_node_actual_shape_eq!(node, expect, i);
  }
}

#[test]
fn move_by1() {
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
  let mut tree = Itree::new(n1);
  tree.insert(nid1, n2);
  tree.insert(nid2, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  info!("n1:{:?}", n1);
  info!("n2:{:?}", n2);
  info!("n3:{:?}", n3);

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
  let mut tree = Itree::new(n1);
  tree.insert(nid1, n2);
  tree.insert(nid2, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  info!("n1:{:?}", n1);
  info!("n2:{:?}", n2);
  info!("n3:{:?}", n3);

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
fn move_to1() {
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
  let mut tree = Itree::new(n1);
  tree.insert(nid1, n2);
  tree.insert(nid2, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  info!("n1:{:?}", n1);
  info!("n2:{:?}", n2);
  info!("n3:{:?}", n3);

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
  let mut tree = Itree::new(n1);
  tree.insert(nid1, n2);
  tree.insert(nid2, n3);

  let n1 = tree.node(nid1).unwrap();
  let n2 = tree.node(nid2).unwrap();
  let n3 = tree.node(nid3).unwrap();
  info!("n1:{:?}", n1);
  info!("n2:{:?}", n2);
  info!("n3:{:?}", n3);

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
