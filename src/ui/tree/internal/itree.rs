//! Internal tree structure that implements the widget tree.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use std::{collections::VecDeque, iter::Iterator};
use tracing::debug;

use crate::cart::{shapes, U16Rect};
use crate::glovar;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};

#[derive(Debug, Default)]
pub struct Itree<T>
where
  T: InodeValue,
{
  // Root node ID.
  root_id: InodeId,
  // Nodes collection, maps from node ID to its node struct.
  nodes: HashMap<InodeId, Mutex<Inode<T>>>,
  // Maps from child ID to its parent ID.
  parent_ids: HashMap<InodeId, InodeId>,
  // Maps from parent ID to its children IDs, the children are sorted by zindex value from lower to higher.
  // For those children share the same zindex value, the one later inserted into the vector will be put in the back, thus it will be rendered later, i.e. it implicitly has a higher priority to show.
  children_ids: HashMap<InodeId, Vec<InodeId>>,
}

#[derive(Debug, Clone)]
/// The pre-order iterator of the tree.
///
/// For each node, it first visits the node itself, then visits all its children.
/// This also follows the order when rendering the widget tree to terminal device.
///
/// By default, the visiting order for the children is from lower z-index to higher, thus the higher z-index ones will cover those lower ones.
pub struct ItreeIterator<'a, T>
where
  T: InodeValue,
{
  tree: &'a Itree<T>,
  order: ItreeIterateOrder,
  queue: VecDeque<&'a Mutex<Inode<T>>>,
}

impl<'a, T> Iterator for ItreeIterator<'a, T>
where
  T: InodeValue,
{
  type Item = &'a Mutex<Inode<T>>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(node) = self.queue.pop_front() {
      let node_guard = node
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap();
      match self.tree.children_ids(node_guard.id()) {
        Some(children_ids) => match self.order {
          ItreeIterateOrder::Ascent => {
            for child_id in children_ids.iter() {
              match self.tree.node(*child_id) {
                Some(child) => {
                  self.queue.push_back(child);
                }
                None => { /* Skip */ }
              }
            }
          }
          ItreeIterateOrder::Descent => {
            for child_id in children_ids.iter().rev() {
              match self.tree.node(*child_id) {
                Some(child) => {
                  self.queue.push_back(child);
                }
                None => { /* Skip */ }
              }
            }
          }
        },
        None => { /* Skip */ }
      }
      return Some(node);
    }
    None
  }
}

impl<'a, T> ItreeIterator<'a, T>
where
  T: InodeValue,
{
  pub fn new(
    tree: &'a Itree<T>,
    order: ItreeIterateOrder,
    start: Option<&'a Mutex<Inode<T>>>,
  ) -> Self {
    let mut queue = VecDeque::new();
    match start {
      Some(start) => queue.push_back(start),
      None => { /* Do nothing */ }
    }
    ItreeIterator { tree, order, queue }
  }
}

#[derive(Debug, Clone)]
/// The iterating order for all the children nodes under the same node.
///
/// * The `Ascent` visits from lower z-index to higher.
/// * The `Descent` visits from higher z-index to lower.
pub enum ItreeIterateOrder {
  Ascent,
  Descent,
}

impl<T> Itree<T>
where
  T: InodeValue,
{
  pub fn new(root_node: Inode<T>) -> Self {
    let root_id = root_node.id();
    let mut nodes = HashMap::new();
    nodes.insert(root_id, Mutex::new(root_node));
    let mut children_ids: HashMap<InodeId, Vec<InodeId>> = HashMap::new();
    children_ids.insert(root_id, vec![]);
    Itree {
      root_id,
      nodes,
      parent_ids: HashMap::new(),
      children_ids,
    }
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  pub fn is_empty(&self) -> bool {
    self.nodes.len() <= 1
  }

  pub fn root_id(&self) -> InodeId {
    self.root_id
  }

  pub fn node_ids(&self) -> Vec<InodeId> {
    self.nodes.keys().copied().collect()
  }

  pub fn parent_id(&self, id: InodeId) -> Option<&InodeId> {
    self.parent_ids.get(&id)
  }

  pub fn children_ids(&self, id: InodeId) -> Option<&Vec<InodeId>> {
    self.children_ids.get(&id)
  }

  pub fn node(&self, id: InodeId) -> Option<&Mutex<Inode<T>>> {
    self.nodes.get(&id)
  }

  pub fn node_mut(&mut self, id: InodeId) -> Option<&mut Mutex<Inode<T>>> {
    self.nodes.get_mut(&id)
  }

  /// Get the iterator.
  ///
  /// By default, it iterates in pre-order iterator which starts from the root.
  /// For the children under the same node, it visits from lower z-index to higher.
  pub fn iter(&self) -> ItreeIterator<T> {
    ItreeIterator::new(
      self,
      ItreeIterateOrder::Ascent,
      Some(self.nodes.get(&self.root_id).unwrap()),
    )
  }

  /// Get the iterator with a specified order.
  pub fn ordered_iter(&self, order: ItreeIterateOrder) -> ItreeIterator<T> {
    ItreeIterator::new(self, order, Some(self.nodes.get(&self.root_id).unwrap()))
  }

  /// Insert a node to the tree, i.e. push it to the children vector of the parent.
  ///
  /// This operation builds the connection between the parent and the inserted child.
  ///
  /// It also sorts the children vector after inserted by the z-index value,
  /// and updates both the inserted child's attributes and all its descendants attributes.
  ///
  /// Below node attributes need to update:
  ///
  /// 1. [`depth`](Inode::depth()): The child depth should be always the parent depth + 1.
  /// 2. [`actual_shape`](Inode::actual_shape()): The child actual shape should be always be clipped by parent's boundaries.
  ///
  /// Fails if:
  ///
  /// 1. The `parent_id` doesn't exist.
  pub fn insert(&mut self, parent_id: InodeId, child_node: Inode<T>) -> Option<&Mutex<Inode<T>>> {
    // Returns `None` if `parent_id` not exists.
    self.nodes.get(&parent_id)?;

    debug!(
      "parent_id:{:?}, node_ids:{:?}, children_ids:{:?}",
      parent_id,
      self.node_ids(),
      self.children_ids
    );
    assert!(
      self.children_ids.contains_key(&parent_id),
      "children_ids {:?} doesn't contains parent_id {:?}",
      self.children_ids,
      parent_id
    );

    // Insert node.
    let child_id = child_node.id();
    let child_zindex = *child_node.zindex();
    self.nodes.insert(child_id, Mutex::new(child_node));
    self.children_ids.insert(child_id, vec![]);

    // Map child ID => parent ID.
    self.parent_ids.insert(child_id, parent_id);
    // Map parent ID => children IDs.
    // It inserts child ID to the `children_ids` vector of the parent, sorted by the z-index.
    // For the children that have the same z-index value, it inserts at the end of those children.
    // debug!("before get higher zindex pos");
    let higher_zindex_pos: Vec<usize> = self
      .children_ids
      .get(&parent_id)
      .unwrap()
      .iter()
      .enumerate()
      .filter(|(_index, cid)| match self.nodes.get(cid) {
        Some(cnode) => {
          *cnode
            .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
            .unwrap()
            .zindex()
            > child_zindex
        }
        None => false,
      })
      .map(|(index, _cid)| index)
      .collect();
    // debug!("after get higher zindex pos");
    match higher_zindex_pos.first() {
      Some(insert_pos) => {
        self
          .children_ids
          .get_mut(&parent_id)
          .unwrap()
          .insert(*insert_pos, child_id);
      }
      None => {
        self
          .children_ids
          .get_mut(&parent_id)
          .unwrap()
          .push(child_id);
      }
    }

    // Create the queue of parent-child ID pairs, to iterate all descendants under the child node.

    // Tuple of (child, parent depth, parent actual shape)
    type ChildAndParentPair<'a, T> = (&'a Mutex<Inode<T>>, usize, U16Rect);

    // debug!("before create que");
    let mut que: VecDeque<ChildAndParentPair<T>> = VecDeque::new();
    {
      let pnode = self.nodes.get(&parent_id).unwrap();
      let pnode_guard = pnode
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap();
      let pnode_depth = *pnode_guard.depth();
      let pnode_actual_shape = *pnode_guard.actual_shape();
      que.push_back((
        self.nodes.get(&child_id).unwrap(),
        pnode_depth,
        pnode_actual_shape,
      ));
    }
    // debug!("after create que");

    // Iterate all descendants, and update their attributes.
    while let Some(child_and_parent) = que.pop_front() {
      let cnode = child_and_parent.0;
      let pnode_depth = child_and_parent.1;
      let pnode_actual_shape = child_and_parent.2;

      {
        // debug!("before update cnode attr: {:?}", cnode);
        let mut cnode_guard = cnode
          .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
          .unwrap();
        let cnode_id = cnode_guard.id();
        let cnode_depth = pnode_depth + 1;
        let cnode_shape = *cnode_guard.shape();
        let cnode_actual_shape = shapes::convert_to_actual_shape(cnode_shape, pnode_actual_shape);

        *cnode_guard.depth_mut() = cnode_depth;
        *cnode_guard.actual_shape_mut() = cnode_actual_shape;
        // debug!("after update cnode attr: {:?}", cnode_id);

        // debug!(
        //   "before push descendant_ids, cnode_id:{:?}, children_ids: {:?}",
        //   cnode_id, self.children_ids
        // );
        match self.children_ids.get(&cnode_id) {
          Some(descendant_ids) => {
            for dnode_id in descendant_ids.iter() {
              // debug!("before push dnode: {:?}", dnode_id);
              match self.nodes.get(dnode_id) {
                Some(dnode) => {
                  que.push_back((dnode, cnode_depth, cnode_actual_shape));
                }
                None => { /* Skip */ }
              }
              // debug!("after push dnode: {:?}", dnode_id);
            }
          }
          None => { /* Skip */ }
        }
      }
    }

    // Return the inserted child
    self.nodes.get(&child_id)
  }

  /// Remove a node by its ID.
  ///
  /// This operation breaks the connection between the removed node and its parent.
  ///
  /// But the relationships between the removed node and its descendants still remains in the tree,
  /// thus once you insert it back in the same tree, its descendants are still connected with the removed node.
  ///
  /// Fails if:
  /// 1. The removed node doesn't exist.
  /// 2. The removed node is the root node.
  pub fn remove(&mut self, id: InodeId) -> Option<Mutex<Inode<T>>> {
    // Cannot remove root node.
    if id == self.root_id {
      return None;
    }
    // Remove child from nodes collection.
    match self.nodes.remove(&id) {
      Some(removed) => {
        // Remove child `id` => parent ID mapping.
        self.parent_ids.remove(&id);
        Some(removed)
      }
      None => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use parking_lot::Mutex;
  use std::sync::Once;
  use tracing::info;

  use crate::cart::{IRect, U16Rect};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::internal::inode::InodeValue;
  use crate::uuid;

  use super::*;

  #[derive(Copy, Clone, Debug, Default)]
  struct Tvalue {
    id: InodeId,
    pub value: usize,
  }

  impl Tvalue {
    pub fn new(value: usize) -> Self {
      Tvalue {
        id: uuid::next(),
        value,
      }
    }
    pub fn value(&self) -> usize {
      self.value
    }
  }

  impl InodeValue for Tvalue {
    fn id(&self) -> InodeId {
      self.id
    }
  }

  // Test node
  type Tnode = Inode<Tvalue>;

  static INIT: Once = Once::new();

  fn assert_node_id_eq(node: &Mutex<Tnode>, id: InodeId) {
    assert!(
      node
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .id()
        == id
    );
  }

  fn print_node(node: &Mutex<Tnode>, name: &str) {
    info!(
      "{}: {:?}",
      name,
      node
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
    );
  }

  fn assert_parent_child_nodes_depth(parent: &Mutex<Tnode>, child: &Mutex<Tnode>) {
    assert_eq!(
      *child
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .depth()
        + 1,
      *parent
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .depth()
    );
  }

  fn assert_node_actual_shape_eq(node: &Mutex<Tnode>, expect: U16Rect) {
    assert_eq!(
      *node
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .actual_shape(),
      expect
    );
  }

  fn assert_node_value_eq(node: &Mutex<Tnode>, expect: usize) {
    assert_eq!(
      node
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .value()
        .value,
      expect
    );
  }

  #[test]
  fn new() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue::new(1);
    let s1 = IRect::new((0, 0), (1, 1));
    let prev_id = uuid::next();
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();
    let tree = Itree::new(n1);

    assert_eq!(prev_id + 1, nid1);
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.root_id(), nid1);
    assert!(tree.parent_id(nid1).is_none());
    assert!(tree.children_ids(nid1).is_some());
    assert!(tree.children_ids(nid1).unwrap().is_empty());

    for node in tree.iter() {
      assert_node_id_eq(node, nid1);
    }

    for node in tree.ordered_iter(ItreeIterateOrder::Descent) {
      assert_node_id_eq(node, nid1);
    }
  }

  #[test]
  fn insert1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue::new(1);
    let s1 = IRect::new((0, 0), (1, 1));
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = Tvalue::new(2);
    let s2 = IRect::new((0, 0), (1, 1));
    let n2 = Tnode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = Tvalue::new(3);
    let s3 = IRect::new((0, 0), (1, 1));
    let n3 = Tnode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = Tvalue::new(4);
    let s4 = IRect::new((0, 0), (1, 1));
    let n4 = Tnode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = Tvalue::new(5);
    let s5 = IRect::new((0, 0), (1, 1));
    let n5 = Tnode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = Tvalue::new(6);
    let s6 = IRect::new((0, 0), (1, 1));
    let n6 = Tnode::new(v6, s6);
    let nid6 = n6.id();

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
    let mut tree = Itree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid1, n3);
    tree.insert(nid2, n4);
    tree.insert(nid2, n5);
    tree.insert(nid3, n6);

    assert!(tree.root_id() == nid1);
    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = tree.node(nid6).unwrap();
    print_node(n1, "n1");
    print_node(n2, "n2");
    print_node(n3, "n3");
    print_node(n4, "n4");
    print_node(n5, "n5");
    print_node(n6, "n6");

    assert_eq!(nid1 + 1, nid2);
    assert_eq!(nid2 + 1, nid3);
    assert_eq!(nid3 + 1, nid4);
    assert_eq!(nid4 + 1, nid5);
    assert_eq!(nid5 + 1, nid6);

    assert_parent_child_nodes_depth(n1, n2);
    assert_parent_child_nodes_depth(n1, n3);
    assert_parent_child_nodes_depth(n2, n4);
    assert_parent_child_nodes_depth(n2, n5);
    assert_parent_child_nodes_depth(n2, n6);
    assert_parent_child_nodes_depth(n3, n6);

    assert_eq!(tree.children_ids(nid1).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid2).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid3).unwrap().len(), 1);
    assert_eq!(tree.children_ids(nid4).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid5).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid6).unwrap().len(), 0);

    let contains_child = |parent_id: InodeId, child_id: InodeId| -> bool {
      match tree.children_ids(parent_id) {
        Some(children_ids) => {
          children_ids
            .iter()
            .filter(|cid| **cid == child_id)
            .collect::<Vec<_>>()
            .len()
            == 1
        }
        None => false,
      }
    };

    assert!(contains_child(nid1, nid2));
    assert!(contains_child(nid1, nid3));
    assert!(!contains_child(nid1, nid4));
    assert!(!contains_child(nid1, nid5));
    assert!(!contains_child(nid1, nid6));

    assert!(contains_child(nid2, nid4));
    assert!(contains_child(nid2, nid5));
    assert!(!contains_child(nid2, nid6));

    assert!(contains_child(nid3, nid6));
    assert!(!contains_child(nid3, nid4));
    assert!(!contains_child(nid3, nid5));
  }

  #[test]
  fn insert2() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = Tvalue::new(2);
    let s2 = IRect::new((0, 0), (15, 15));
    let n2 = Tnode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = Tvalue::new(3);
    let s3 = IRect::new((10, 10), (18, 19));
    let n3 = Tnode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = Tvalue::new(4);
    let s4 = IRect::new((3, 5), (20, 14));
    let n4 = Tnode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = Tvalue::new(5);
    let s5 = IRect::new((-3, -5), (10, 20));
    let n5 = Tnode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = Tvalue::new(6);
    let s6 = IRect::new((3, 6), (6, 10));
    let n6 = Tnode::new(v6, s6);
    let nid6 = n6.id();

    let v7 = Tvalue::new(7);
    let s7 = IRect::new((3, 6), (15, 25));
    let n7 = Tnode::new(v7, s7);
    let nid7 = n7.id();

    let v8 = Tvalue::new(8);
    let s8 = IRect::new((-1, -2), (2, 1));
    let n8 = Tnode::new(v8, s8);
    let nid8 = n8.id();

    let v9 = Tvalue::new(9);
    let s9 = IRect::new((5, 6), (9, 8));
    let n9 = Tnode::new(v9, s9);
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
    print_node(n1, "n1");
    print_node(n2, "n2");
    print_node(n3, "n3");
    print_node(n4, "n4");
    print_node(n5, "n5");
    print_node(n6, "n6");
    print_node(n7, "n7");
    print_node(n8, "n8");
    print_node(n9, "n9");

    assert_eq!(nid1 + 1, nid2);
    assert_eq!(nid2 + 1, nid3);
    assert_eq!(nid3 + 1, nid4);
    assert_eq!(nid4 + 1, nid5);
    assert_eq!(nid5 + 1, nid6);
    assert_eq!(nid6 + 1, nid7);
    assert_eq!(nid7 + 1, nid8);
    assert_eq!(nid8 + 1, nid9);

    assert_parent_child_nodes_depth(n1, n2);
    assert_parent_child_nodes_depth(n1, n3);
    assert_parent_child_nodes_depth(n2, n4);
    assert_parent_child_nodes_depth(n2, n5);
    assert_parent_child_nodes_depth(n2, n6);
    assert_parent_child_nodes_depth(n3, n6);
    assert_parent_child_nodes_depth(n5, n7);
    assert_parent_child_nodes_depth(n7, n8);
    assert_parent_child_nodes_depth(n7, n9);

    assert_eq!(tree.children_ids(nid1).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid2).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid3).unwrap().len(), 1);
    assert_eq!(tree.children_ids(nid4).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid5).unwrap().len(), 1);
    assert_eq!(tree.children_ids(nid6).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid7).unwrap().len(), 2);
    assert_eq!(tree.children_ids(nid8).unwrap().len(), 0);
    assert_eq!(tree.children_ids(nid9).unwrap().len(), 0);

    let contains_child = |parent_id: InodeId, child_id: InodeId| -> bool {
      match tree.children_ids(parent_id) {
        Some(children_ids) => {
          children_ids
            .iter()
            .filter(|cid| **cid == child_id)
            .collect::<Vec<_>>()
            .len()
            == 1
        }
        None => false,
      }
    };

    assert!(contains_child(nid1, nid2));
    assert!(contains_child(nid1, nid3));
    assert!(!contains_child(nid1, nid4));
    assert!(!contains_child(nid1, nid5));
    assert!(!contains_child(nid1, nid7));

    assert!(contains_child(nid2, nid4));
    assert!(contains_child(nid2, nid5));
    assert!(!contains_child(nid2, nid7));

    assert!(contains_child(nid3, nid7));
    assert!(!contains_child(nid3, nid4));
    assert!(!contains_child(nid3, nid5));

    assert!(contains_child(nid5, nid7));
    assert!(contains_child(nid7, nid8));
    assert!(contains_child(nid7, nid9));
  }

  #[test]
  fn shape1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = Tvalue::new(2);
    let s2 = IRect::new((0, 0), (15, 15));
    let us2 = U16Rect::new((0, 0), (15, 15));
    let n2 = Tnode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = Tvalue::new(3);
    let s3 = IRect::new((10, 10), (18, 19));
    let us3 = U16Rect::new((10, 10), (18, 19));
    let n3 = Tnode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = Tvalue::new(4);
    let s4 = IRect::new((3, 5), (20, 14));
    let us4 = U16Rect::new((3, 5), (15, 14));
    let n4 = Tnode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = Tvalue::new(5);
    let s5 = IRect::new((-3, -5), (10, 20));
    let us5 = U16Rect::new((0, 0), (10, 15));
    let n5 = Tnode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = Tvalue::new(6);
    let s6 = IRect::new((3, 6), (6, 10));
    let us6 = U16Rect::new((13, 16), (16, 19));
    let n6 = Tnode::new(v6, s6);
    let nid6 = n6.id();

    let v7 = Tvalue::new(7);
    let s7 = IRect::new((3, 6), (15, 25));
    let us7 = U16Rect::new((3, 6), (10, 15));
    let n7 = Tnode::new(v7, s7);
    let nid7 = n7.id();

    let v8 = Tvalue::new(8);
    let s8 = IRect::new((-1, -2), (2, 1));
    let us8 = U16Rect::new((3, 6), (5, 7));
    let n8 = Tnode::new(v8, s8);
    let nid8 = n8.id();

    let v9 = Tvalue::new(9);
    let s9 = IRect::new((5, 6), (9, 8));
    let us9 = U16Rect::new((8, 12), (10, 14));
    let n9 = Tnode::new(v9, s9);
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
    print_node(n1, "n1");
    print_node(n2, "n2");
    print_node(n3, "n3");
    print_node(n4, "n4");
    print_node(n5, "n5");
    print_node(n6, "n6");
    print_node(n7, "n7");
    print_node(n8, "n8");
    print_node(n9, "n9");

    let expects = vec![us1, us2, us3, us4, us5, us6, us7, us8, us9];
    let nodes = vec![n1, n2, n3, n4, n5, n6, n7, n8, n9];
    for i in 0..9 {
      let expect = expects[i];
      assert_node_actual_shape_eq(nodes[i], expect);
    }
  }

  #[test]
  fn shape2() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = Tvalue::new(2);
    let s2 = IRect::new((0, 0), (20, 20));
    let us2 = U16Rect::new((0, 0), (20, 20));
    let n2 = Tnode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = Tvalue::new(3);
    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = Tnode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = Tvalue::new(4);
    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = Tnode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = Tvalue::new(5);
    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((0, 0), (15, 20));
    let n5 = Tnode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = Tvalue::new(6);
    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((8, 13), (15, 20));
    let n6 = Tnode::new(v6, s6);
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
    print_node(n1, "n1");
    print_node(n2, "n2");
    print_node(n3, "n3");
    print_node(n4, "n4");
    print_node(n5, "n5");
    print_node(n6, "n6");

    let expects = vec![us1, us2, us3, us4, us5, us6];
    let nodes = vec![n1, n2, n3, n4, n5, n6];
    for i in 0..9 {
      let expect = expects[i];
      let node = &nodes[i];
      assert_node_actual_shape_eq(node, expect);
    }
  }

  #[test]
  fn push1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let shape = IRect::new((0, 0), (10, 10));
    let nodes: Vec<Tnode> = vec![1, 2, 3, 4, 5]
      .iter()
      .map(|value| Tvalue::new(*value as usize))
      .map(|tv| Tnode::new(tv, shape))
      .collect::<Vec<Tnode>>();
    let nodes_ids: Vec<InodeId> = nodes.iter().map(|n| n.id()).collect();

    /*
     * The tree looks like:
     * ```
     *             n1
     *         /        \
     *       n2, n3, n4, n5
     * ```
     */
    let mut tree = Itree::new(nodes[0].clone());
    for i in 1..5 {
      tree.insert(nodes_ids[0], nodes[i].clone());
    }

    assert!(tree.root_id() == nodes_ids[0]);
    assert!(tree.children_ids(nodes_ids[0]).unwrap().len() == 4);
    assert!(!tree.children_ids(nodes_ids[0]).unwrap().is_empty());
    for i in 1..5 {
      assert!(tree.children_ids(nodes_ids[i]).unwrap().len() == 0);
      assert!(tree.children_ids(nodes_ids[i]).unwrap().is_empty());
    }

    for i in 0..5 {
      assert_node_value_eq(tree.node(i).unwrap(), i);
    }

    let first1 = tree.children_ids(nodes_ids[0]).unwrap().first();
    assert!(first1.is_some());
    assert_eq!(*first1.unwrap(), nodes_ids[1]);

    let last1 = tree.children_ids(nodes_ids[0]).unwrap().last();
    assert!(last1.is_some());
    assert_eq!(*last1.unwrap(), nodes_ids[4]);

    for i in 1..5 {
      let first = tree.children_ids(nodes_ids[i]).unwrap().first();
      let last = tree.children_ids(nodes_ids[i]).unwrap().last();
      assert!(first.is_none());
      assert!(last.is_none());
    }
  }

  fn make_tree(n: usize) -> (Vec<InodeId>, Itree<Tvalue>) {
    let mut value = 1;
    let mut node_ids: Vec<InodeId> = vec![];

    let v = Tvalue::new(value);
    value += 1;
    let s = IRect::new((0, 0), (10, 10));
    let root = Tnode::new(v, s);
    let root_id = root.id();
    node_ids.push(root_id);

    let mut tree = Itree::new(root);
    for _ in 1..n {
      let v = Tvalue::new(value);
      value += 1;
      let node = Tnode::new(v, s);
      let node_id = node.id();
      tree.insert(root_id, node);
      node_ids.push(node_id);
    }

    (node_ids, tree)
  }

  #[test]
  fn remove1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let (node_ids, mut tree) = make_tree(5);
    let remove0 = tree.remove(node_ids[0]);
    let remove2 = tree.remove(node_ids[2]);
    let remove4 = tree.remove(node_ids[4]);

    assert!(remove0.is_none());
    assert!(remove2.is_some());
    assert_node_value_eq(&remove2.unwrap(), 3);
    assert!(remove4.is_some());
    assert_node_value_eq(&remove4.unwrap(), 5);

    let remove1 = tree.remove(node_ids[1]);
    let remove3 = tree.remove(node_ids[3]);

    // 1,2,(3),4,(5)
    assert!(remove1.is_some());
    assert_node_value_eq(&remove1.unwrap(), 2);
    assert!(remove3.is_some());
    assert_node_value_eq(&remove3.unwrap(), 4);
  }

  #[test]
  fn get1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = Tvalue::new(2);
    let s2 = IRect::new((0, 0), (15, 15));
    let n2 = Tnode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = Tvalue::new(3);
    let s3 = IRect::new((10, 10), (18, 19));
    let n3 = Tnode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = Tvalue::new(4);
    let s4 = IRect::new((3, 5), (20, 14));
    let n4 = Tnode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = Tvalue::new(5);
    let s5 = IRect::new((-3, -5), (10, 20));
    let n5 = Tnode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = Tvalue::new(6);
    let s6 = IRect::new((3, 6), (6, 10));
    let n6 = Tnode::new(v6, s6);
    let nid6 = n6.id();

    let v7 = Tvalue::new(7);
    let s7 = IRect::new((3, 6), (15, 25));
    let n7 = Tnode::new(v7, s7);
    let nid7 = n7.id();

    let v8 = Tvalue::new(8);
    let s8 = IRect::new((-1, -2), (2, 1));
    let n8 = Tnode::new(v8, s8);
    let nid8 = n8.id();

    let v9 = Tvalue::new(9);
    let s9 = IRect::new((5, 6), (9, 8));
    let n9 = Tnode::new(v9, s9);
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
    print_node(n1, "n1");
    print_node(n2, "n2");
    print_node(n3, "n3");
    print_node(n4, "n4");
    print_node(n5, "n5");
    print_node(n6, "n6");
    print_node(n7, "n7");
    print_node(n8, "n8");
    print_node(n9, "n9");
  }

  #[test]
  fn get2() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue::new(1);
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = Tvalue::new(2);
    let s2 = IRect::new((0, 0), (20, 20));
    let us2 = U16Rect::new((0, 0), (20, 20));
    let n2 = Tnode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = Tvalue::new(3);
    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = Tnode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = Tvalue::new(4);
    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = Tnode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = Tvalue::new(5);
    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((0, 0), (15, 20));
    let n5 = Tnode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = Tvalue::new(6);
    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((8, 13), (15, 20));
    let n6 = Tnode::new(v6, s6);
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
    // print_node(n1, "n1");
    // print_node(n2, "n2");
    // print_node(n3, "n3");
    // print_node(n4, "n4");
    // print_node(n5, "n5");
    // print_node(n6, "n6");

    let expects = vec![us1, us2, us3, us4, us5, us6];
    let nodes = vec![n1, n2, n3, n4, n5, n6];
    for i in 0..9 {
      let expect = expects[i];
      let node = &nodes[i];
      assert_node_actual_shape_eq(node, expect);
    }
  }
}
