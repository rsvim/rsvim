//! Internal tree structure that implements the widget tree.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::{collections::VecDeque, iter::Iterator};

use crate::cart::shapes;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};

#[derive(Debug, Clone, Default)]
pub struct Itree<T>
where
  T: InodeValue,
{
  // Root node ID.
  root_id: InodeId,
  // Nodes collection, maps from node ID to its node struct.
  nodes: HashMap<InodeId, RefCell<Inode<T>>>,
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
  queue: VecDeque<&'a RefCell<Inode<T>>>,
}

impl<'a, T> Iterator for ItreeIterator<'a, T>
where
  T: InodeValue,
{
  type Item = &'a RefCell<Inode<T>>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(node) = self.queue.pop_front() {
      match self.tree.children_ids(node.borrow().id()) {
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
    start: Option<&'a RefCell<Inode<T>>>,
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
    nodes.insert(root_id, RefCell::new(root_node));
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

  pub fn root_id(&self) -> InodeId {
    self.root_id
  }

  pub fn parent_id(&self, id: InodeId) -> Option<&InodeId> {
    self.parent_ids.get(&id)
  }

  pub fn children_ids(&self, id: InodeId) -> Option<&Vec<InodeId>> {
    self.children_ids.get(&id)
  }

  pub fn node(&self, id: InodeId) -> Option<&RefCell<Inode<T>>> {
    self.nodes.get(&id)
  }

  pub fn node_mut(&mut self, id: InodeId) -> Option<&mut RefCell<Inode<T>>> {
    self.nodes.get_mut(&id)
  }

  /// Assert the `node` exists in the tree.
  ///
  /// # Panics
  ///
  /// Panics when the `node` doesn't exist.
  fn assert_exists(&self, node: InodeArc<T>) {
    assert!(
      self.root.is_some(),
      "Doesn't have a root node when assert the node exists"
    );
    let node = node.lock();
    let node_id = node.borrow().id();
    let root_node = self.root.clone().unwrap();
    let node2 = root_node.lock().borrow().get_descendant(node_id);
    assert!(node2.is_some(), "Missing node {} in the tree", node_id);
    let node2_id = node2.unwrap().lock().borrow().id();
    assert!(
      node2_id == node_id,
      "Node ID {} not match in the tree",
      node_id
    );
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
  /// 2. [`actual_shape`](Inode::actual_shape()): The child actual shape should be always be clipped by parent's boundries.
  ///
  /// Fails if:
  ///
  /// 1. The `parent_id` doesn't exist.
  pub fn insert(&mut self, parent_id: InodeId, child_node: Inode<T>) -> Option<&RefCell<Inode<T>>> {
    if self.nodes.get(&parent_id).is_none() {
      return None;
    }

    // Insert node.
    let child_id = child_node.id();
    let child_zindex = child_node.zindex();
    self.nodes.insert(child_id, RefCell::new(child_node));

    // Map child ID => parent ID.
    self.parent_ids.insert(child_id, parent_id);
    // Map parent ID => children IDs.
    // It inserts child ID to the `children_ids` vector of the parent, sorted by the z-index.
    // For the children that have the same z-index value, it inserts at the end of those children.
    let higher_zindex_pos: Vec<usize> = self
      .children_ids
      .get(&parent_id)
      .unwrap()
      .iter()
      .enumerate()
      .filter(|(_index, cid)| match self.nodes.get(&cid) {
        Some(cnode) => cnode.borrow().zindex() > child_zindex,
        None => false,
      })
      .map(|(index, _cid)| index)
      .collect();
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
    let mut que: VecDeque<(&RefCell<Inode<T>>, &RefCell<Inode<T>>)> = VecDeque::new();
    que.push_back((
      self.nodes.get(&parent_id).unwrap(),
      self.nodes.get(&child_id).unwrap(),
    ));

    // Iterate all descendants, and update their attributes.
    while let Some(parent_and_child) = que.pop_front() {
      let pnode = parent_and_child.0;
      let cnode = parent_and_child.1;
      *cnode.borrow_mut().depth_mut() = pnode.borrow().depth() + 1;
      *cnode.borrow_mut().actual_shape_mut() =
        shapes::convert_to_actual_shape(*cnode.borrow().shape(), *pnode.borrow().actual_shape());

      match self.children_ids.get(&cnode.borrow().id()) {
        Some(descendant_ids) => {
          for descendant_id in descendant_ids.iter() {
            match self.nodes.get(&descendant_id) {
              Some(dnode) => {
                que.push_back((cnode, dnode));
              }
              None => { /* Skip */ }
            }
          }
        }
        None => { /* Skip */ }
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
  pub fn remove(&mut self, id: InodeId) -> Option<Inode<T>> {
    // Cannot remove root node.
    if id == self.root_id {
      return None;
    }
    // Remove child from nodes collection.
    match self.nodes.remove(&id) {
      Some(removed) => {
        // Remove child `id` => parent ID mapping.
        self.parent_ids.remove(&id);
        Some(removed.take())
      }
      None => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Once;

  use parking_lot::ReentrantMutexGuard;
  use tracing::info;

  use crate::cart::{IRect, U16Rect};
  use crate::test::log::init as test_log_init;
  use crate::ui::tree::internal::inode::InodeValue;
  use crate::uuid;

  use super::*;

  #[derive(Copy, Clone, Debug, Default)]
  struct Tvalue {
    pub value: usize,
  }

  impl InodeValue for Tvalue {
    fn id(&self) -> InodeId {
      self.value
    }
  }

  // Test node
  type Tnode = Inode<Tvalue>;

  static INIT: Once = Once::new();

  #[test]
  fn new() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue { value: 1 };
    let s1 = IRect::new((0, 0), (1, 1));
    let us1 = U16Rect::new((0, 0), (1, 1));
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
      assert!(node.borrow().id() == nid1);
    }

    for node in tree.ordered_iter(ItreeIterateOrder::Descent) {
      assert!(node.borrow().id() == nid1);
    }
  }

  #[test]
  fn insert1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue { value: 1 };
    let s1 = IRect::new((0, 0), (1, 1));
    let us1 = U16Rect::new((0, 0), (1, 1));
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = Tvalue { value: 2 };
    let s2 = IRect::new((0, 0), (1, 1));
    let us2 = U16Rect::new((0, 0), (1, 1));
    let n2 = Tnode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = Tvalue { value: 3 };
    let s3 = IRect::new((0, 0), (1, 1));
    let us3 = U16Rect::new((0, 0), (1, 1));
    let n3 = Tnode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = Tvalue { value: 4 };
    let s4 = IRect::new((0, 0), (1, 1));
    let us4 = U16Rect::new((0, 0), (1, 1));
    let n4 = Tnode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = Tvalue { value: 5 };
    let s5 = IRect::new((0, 0), (1, 1));
    let us5 = U16Rect::new((0, 0), (1, 1));
    let n5 = Tnode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = Tvalue { value: 6 };
    let s6 = IRect::new((0, 0), (1, 1));
    let us6 = U16Rect::new((0, 0), (1, 1));
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

    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = tree.node(nid6).unwrap();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());

    assert_eq!(nid1 + 1, nid2);
    assert_eq!(nid2 + 1, nid3);
    assert_eq!(nid3 + 1, nid4);
    assert_eq!(nid4 + 1, nid5);
    assert_eq!(nid5 + 1, nid6);

    assert_eq!(*n1.borrow().depth() + 1, *n2.borrow().depth());
    assert_eq!(*n1.borrow().depth() + 1, *n3.borrow().depth());
    assert_eq!(*n2.borrow().depth() + 1, *n4.borrow().depth());
    assert_eq!(*n2.borrow().depth() + 1, *n5.borrow().depth());
    assert_eq!(*n2.borrow().depth() + 1, *n6.borrow().depth());
    assert_eq!(*n3.borrow().depth() + 1, *n6.borrow().depth());

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

    let v1 = Tvalue { value: 1 };
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(v1, s1);
    let nid1 = n1.id();

    let v2 = Tvalue { value: 2 };
    let s2 = IRect::new((0, 0), (15, 15));
    let us2 = U16Rect::new((0, 0), (15, 15));
    let n2 = Tnode::new(v2, s2);
    let nid2 = n2.id();

    let v3 = Tvalue { value: 3 };
    let s3 = IRect::new((10, 10), (18, 19));
    let us3 = U16Rect::new((10, 10), (18, 19));
    let n3 = Tnode::new(v3, s3);
    let nid3 = n3.id();

    let v4 = Tvalue { value: 4 };
    let s4 = IRect::new((3, 5), (20, 14));
    let us4 = U16Rect::new((3, 5), (15, 14));
    let n4 = Tnode::new(v4, s4);
    let nid4 = n4.id();

    let v5 = Tvalue { value: 5 };
    let s5 = IRect::new((-3, -5), (10, 20));
    let us5 = U16Rect::new((0, 0), (10, 15));
    let n5 = Tnode::new(v5, s5);
    let nid5 = n5.id();

    let v6 = Tvalue { value: 6 };
    let s6 = IRect::new((3, 6), (6, 10));
    let us6 = U16Rect::new((13, 16), (16, 19));
    let n6 = Tnode::new(v6, s6);
    let nid6 = n6.id();

    let v7 = Tvalue { value: 7 };
    let s7 = IRect::new((3, 6), (15, 25));
    let us7 = U16Rect::new((3, 6), (10, 15));
    let n7 = Tnode::new(v7, s7);
    let nid7 = n7.id();

    let v8 = Tvalue { value: 8 };
    let s8 = IRect::new((-1, -2), (2, 1));
    let us8 = U16Rect::new((3, 6), (5, 7));
    let n8 = Tnode::new(v8, s8);
    let nid8 = n8.id();

    let v9 = Tvalue { value: 9 };
    let s9 = IRect::new((5, 6), (9, 8));
    let us9 = U16Rect::new((8, 12), (10, 14));
    let n9 = Tnode::new(v9, s9);
    let nid9 = n9.id();

    /**
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
     **/
    let mut tree = Itree::new(n1);
    tree.insert(nid1, n2);
    tree.insert(nid1, n3);
    tree.insert(nid2, n4);
    tree.insert(nid2, n5);
    tree.insert(nid3, n6);
    tree.insert(nid5, n7);
    tree.insert(nid7, n8);
    tree.insert(nid7, n9);

    let n1 = tree.node(nid1).unwrap();
    let n2 = tree.node(nid2).unwrap();
    let n3 = tree.node(nid3).unwrap();
    let n4 = tree.node(nid4).unwrap();
    let n5 = tree.node(nid5).unwrap();
    let n6 = n6.lock();
    let n7 = n7.lock();
    let n8 = n8.lock();
    let n9 = n9.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());
    info!("n7:{:?}", n7.borrow());
    info!("n8:{:?}", n8.borrow());
    info!("n9:{:?}", n9.borrow());

    assert_eq!(nid1 + 1, nid2);
    assert_eq!(nid2 + 1, nid3);
    assert_eq!(nid3 + 1, nid4);
    assert_eq!(nid4 + 1, nid5);
    assert_eq!(nid5 + 1, nid6);
    assert_eq!(nid6 + 1, nid7);
    assert_eq!(nid7 + 1, nid8);
    assert_eq!(nid8 + 1, nid9);

    assert_eq!(n1.borrow().depth() + 1, n2.borrow().depth());
    assert_eq!(n1.borrow().depth() + 1, n3.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n4.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n5.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n6.borrow().depth());
    assert_eq!(n3.borrow().depth() + 1, n6.borrow().depth());
    assert_eq!(n5.borrow().depth() + 1, n7.borrow().depth());
    assert_eq!(n7.borrow().depth() + 1, n8.borrow().depth());
    assert_eq!(n7.borrow().depth() + 1, n9.borrow().depth());

    assert_eq!(n1.borrow().children().len(), 2);
    assert_eq!(n2.borrow().children().len(), 2);
    assert_eq!(n3.borrow().children().len(), 1);
    assert_eq!(n4.borrow().children().len(), 0);
    assert_eq!(n5.borrow().children().len(), 1);
    assert_eq!(n6.borrow().children().len(), 0);
    assert_eq!(n7.borrow().children().len(), 2);
    assert_eq!(n8.borrow().children().len(), 0);
    assert_eq!(n9.borrow().children().len(), 0);

    let contains_node = |parent: &ReentrantMutexGuard<RefCell<Tnode>>, child_id: usize| -> bool {
      parent
        .borrow()
        .children()
        .iter()
        .filter(|c| c.lock().borrow().id() == child_id)
        .collect::<Vec<_>>()
        .len()
        == 1
    };

    assert!(contains_node(&n1, nid2));
    assert!(contains_node(&n1, nid3));
    assert!(!contains_node(&n1, nid4));
    assert!(!contains_node(&n1, nid5));
    assert!(!contains_node(&n1, nid7));

    assert!(contains_node(&n2, nid4));
    assert!(contains_node(&n2, nid5));
    assert!(!contains_node(&n2, nid7));

    assert!(contains_node(&n3, nid7));
    assert!(!contains_node(&n3, nid4));
    assert!(!contains_node(&n3, nid5));

    assert!(contains_node(&n5, nid7));
    assert!(contains_node(&n7, nid8));
    assert!(contains_node(&n7, nid9));
  }

  #[test]
  fn shape1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue { value: 1 };
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(None, v1, s1);
    let nid1 = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = Tvalue { value: 2 };
    let s2 = IRect::new((0, 0), (15, 15));
    let us2 = U16Rect::new((0, 0), (15, 15));
    let n2 = Tnode::new(None, v2, s2);
    let nid2 = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = Tvalue { value: 3 };
    let s3 = IRect::new((10, 10), (18, 19));
    let us3 = U16Rect::new((10, 10), (18, 19));
    let n3 = Tnode::new(None, v3, s3);
    let nid3 = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = Tvalue { value: 4 };
    let s4 = IRect::new((3, 5), (20, 14));
    let us4 = U16Rect::new((3, 5), (15, 14));
    let n4 = Tnode::new(None, v4, s4);
    let nid4 = n4.id();
    let n4 = Tnode::to_arc(n4);

    let v5 = Tvalue { value: 5 };
    let s5 = IRect::new((-3, -5), (10, 20));
    let us5 = U16Rect::new((0, 0), (10, 15));
    let n5 = Tnode::new(None, v5, s5);
    let nid5 = n5.id();
    let n5 = Tnode::to_arc(n5);

    let v6 = Tvalue { value: 6 };
    let s6 = IRect::new((3, 6), (6, 10));
    let us6 = U16Rect::new((13, 16), (16, 19));
    let n6 = Tnode::new(None, v6, s6);
    let nid6 = n6.id();
    let n6 = Tnode::to_arc(n6);

    let v7 = Tvalue { value: 7 };
    let s7 = IRect::new((3, 6), (15, 25));
    let us7 = U16Rect::new((3, 6), (10, 15));
    let n7 = Tnode::new(None, v7, s7);
    let nid7 = n7.id();
    let n7 = Tnode::to_arc(n7);

    let v8 = Tvalue { value: 8 };
    let s8 = IRect::new((-1, -2), (2, 1));
    let us8 = U16Rect::new((3, 6), (5, 7));
    let n8 = Tnode::new(None, v8, s8);
    let nid8 = n8.id();
    let n8 = Tnode::to_arc(n8);

    let v9 = Tvalue { value: 9 };
    let s9 = IRect::new((5, 6), (9, 8));
    let us9 = U16Rect::new((8, 12), (10, 14));
    let n9 = Tnode::new(None, v9, s9);
    let nid9 = n9.id();
    let n9 = Tnode::to_arc(n9);

    /**
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
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n2.clone(), n5.clone());
    Inode::push(n3.clone(), n6.clone());
    Inode::push(n5.clone(), n7.clone());
    Inode::push(n7.clone(), n8.clone());
    Inode::push(n7.clone(), n9.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    let n3 = n3.lock();
    let n4 = n4.lock();
    let n5 = n5.lock();
    let n6 = n6.lock();
    let n7 = n7.lock();
    let n8 = n8.lock();
    let n9 = n9.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());
    info!("n7:{:?}", n7.borrow());
    info!("n8:{:?}", n8.borrow());
    info!("n9:{:?}", n9.borrow());

    let expects = vec![us1, us2, us3, us4, us5, us6, us7, us8, us9];
    let nodes = vec![n1, n2, n3, n4, n5, n6, n7, n8, n9];
    for i in 0..9 {
      let expect = expects[i];
      let node = &nodes[i];
      let actual = node.borrow().actual_shape();
      assert_eq!(expect, actual);
    }
  }

  #[test]
  fn shape2() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue { value: 1 };
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(None, v1, s1);
    let nid1 = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = Tvalue { value: 2 };
    let s2 = IRect::new((0, 0), (20, 20));
    let us2 = U16Rect::new((0, 0), (20, 20));
    let n2 = Tnode::new(None, v2, s2);
    let nid2 = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = Tvalue { value: 3 };
    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = Tnode::new(None, v3, s3);
    let nid3 = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = Tvalue { value: 4 };
    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = Tnode::new(None, v4, s4);
    let nid4 = n4.id();
    let n4 = Tnode::to_arc(n4);

    let v5 = Tvalue { value: 5 };
    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((0, 0), (15, 20));
    let n5 = Tnode::new(None, v5, s5);
    let nid5 = n5.id();
    let n5 = Tnode::to_arc(n5);

    let v6 = Tvalue { value: 5 };
    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((8, 13), (15, 20));
    let n6 = Tnode::new(None, v6, s6);
    let nid6 = n6.id();
    let n6 = Tnode::to_arc(n6);

    /**
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
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n4.clone(), n5.clone());
    Inode::push(n5.clone(), n6.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    let n3 = n3.lock();
    let n4 = n4.lock();
    let n5 = n5.lock();
    let n6 = n6.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());

    let expects = vec![us1, us2, us3, us4, us5, us6];
    let nodes = vec![n1, n2, n3, n4, n5, n6];
    for i in 0..9 {
      let expect = expects[i];
      let node = &nodes[i];
      let actual = node.borrow().actual_shape();
      assert_eq!(expect, actual);
    }
  }

  #[test]
  fn push1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue { value: 1 };
    let s1 = IRect::new((0, 0), (10, 10));
    let n1 = Tnode::new(None, v1, s1);
    let n1 = Tnode::to_arc(n1);

    let v2 = Tvalue { value: 2 };
    let s2 = IRect::new((0, 0), (10, 10));
    let n2 = Tnode::new(None, v2, s2);
    let n2 = Tnode::to_arc(n2);

    let v3 = Tvalue { value: 3 };
    let s3 = IRect::new((0, 0), (10, 10));
    let n3 = Tnode::new(None, v3, s3);
    let n3 = Tnode::to_arc(n3);

    let v4 = Tvalue { value: 4 };
    let s4 = IRect::new((0, 0), (10, 10));
    let n4 = Tnode::new(None, v4, s4);
    let n4 = Tnode::to_arc(n4);

    let v5 = Tvalue { value: 5 };
    let s5 = IRect::new((0, 0), (10, 10));
    let n5 = Tnode::new(None, v5, s5);
    let n5 = Tnode::to_arc(n5);

    /**
     * The tree looks like:
     * ```
     *             n1
     *         /        \
     *       n2, n3, n4, n5
     * ```
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n1.clone(), n4.clone());
    Inode::push(n1.clone(), n5.clone());

    let n1 = n1.lock();
    let n1 = n1.borrow();
    let n2 = n2.lock();
    let n2 = n2.borrow();
    let n3 = n3.lock();
    let n3 = n3.borrow();
    let n4 = n4.lock();
    let n4 = n4.borrow();
    let n5 = n5.lock();
    let n5 = n5.borrow();

    assert_eq!(n1.children().len(), 3);
    assert!(!n1.children().is_empty());
    assert_eq!(n2.children().len(), 0);
    assert!(n2.children().is_empty());
    assert_eq!(n3.children().len(), 0);
    assert!(n3.children().is_empty());
    assert_eq!(n4.children().len(), 0);
    assert!(n4.children().is_empty());
    assert_eq!(n5.children().len(), 0);
    assert!(n5.children().is_empty());

    for (i, c) in n1.children().iter().enumerate() {
      assert_eq!(i + 2, c.lock().borrow().value().value);
    }

    let first1 = n1.children().first();
    assert!(first1.is_some());
    assert_eq!(first1.unwrap().lock().borrow().value().value, 2);

    let last1 = n1.children().last();
    assert!(last1.is_some());
    assert_eq!(last1.unwrap().lock().borrow().value().value, 4);

    assert!(n2.children().first().is_none());
    assert!(n2.children().last().is_none());
    assert!(n3.children().first().is_none());
    assert!(n3.children().last().is_none());
    assert!(n4.children().first().is_none());
    assert!(n4.children().last().is_none());
    assert!(n5.children().first().is_none());
    assert!(n5.children().last().is_none());
  }

  fn make_nodes(n: crate::ui::tree::internal::inode::InodeArc) -> Vec<InodeArc<Tvalue>> {
    let mut value = 1;
    let mut result: Vec<InodeArc<Tvalue>> = vec![];

    let v = Tvalue { value };
    value += 1;
    let s = IRect::new((0, 0), (10, 10));
    let root = Tnode::new(None, v, s);
    let root = Tnode::to_arc(root);
    result.push(root.clone());

    for i in 1..n {
      let v = Tvalue { value };
      value += 1;
      let n = Tnode::new(None, v, s);
      let n = Tnode::to_arc(n);
      Inode::push(root.clone(), n.clone());
      result.push(n.clone());
    }

    result
  }

  #[test]
  fn pop1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let nodes = make_nodes(5);
    let root = nodes[0].clone();
    let remove1 = root.lock().borrow_mut().remove(1);
    let remove2 = root.lock().borrow_mut().remove(2);

    assert!(remove1.is_some());
    assert!(remove1.unwrap().lock().borrow().value().value == 3);
    assert!(remove2.is_some());
    assert!(remove2.unwrap().lock().borrow().value().value == 5);

    let pop3 = root.lock().borrow_mut().pop();
    let pop4 = root.lock().borrow_mut().pop();
    let pop5 = root.lock().borrow_mut().pop();

    // 1,2,(3),4,(5)
    assert!(pop3.is_some());
    assert!(pop3.unwrap().lock().borrow().value().value == 4);
    assert!(pop4.is_some());
    assert!(pop4.unwrap().lock().borrow().value().value == 2);
    assert!(pop5.is_none());
  }

  #[test]
  fn get1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue { value: 1 };
    let s1 = IRect::new((0, 0), (20, 20));
    let n1 = Tnode::new(None, v1, s1);
    let n1 = Tnode::to_arc(n1);

    let v2 = Tvalue { value: 2 };
    let s2 = IRect::new((0, 0), (15, 15));
    let n2 = Tnode::new(None, v2, s2);
    let n2 = Tnode::to_arc(n2);

    let v3 = Tvalue { value: 3 };
    let s3 = IRect::new((10, 10), (18, 19));
    let n3 = Tnode::new(None, v3, s3);
    let n3 = Tnode::to_arc(n3);

    let v4 = Tvalue { value: 4 };
    let s4 = IRect::new((3, 5), (20, 14));
    let n4 = Tnode::new(None, v4, s4);
    let n4 = Tnode::to_arc(n4);

    let v5 = Tvalue { value: 5 };
    let s5 = IRect::new((-3, -5), (10, 20));
    let n5 = Tnode::new(None, v5, s5);
    let n5 = Tnode::to_arc(n5);

    let v6 = Tvalue { value: 6 };
    let s6 = IRect::new((3, 6), (6, 10));
    let n6 = Tnode::new(None, v6, s6);
    let n6 = Tnode::to_arc(n6);

    let v7 = Tvalue { value: 7 };
    let s7 = IRect::new((3, 6), (15, 25));
    let n7 = Tnode::new(None, v7, s7);
    let n7 = Tnode::to_arc(n7);

    let v8 = Tvalue { value: 8 };
    let s8 = IRect::new((-1, -2), (2, 1));
    let n8 = Tnode::new(None, v8, s8);
    let n8 = Tnode::to_arc(n8);

    let v9 = Tvalue { value: 9 };
    let s9 = IRect::new((5, 6), (9, 8));
    let n9 = Tnode::new(None, v9, s9);
    let n9 = Tnode::to_arc(n9);

    /**
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
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n2.clone(), n5.clone());
    Inode::push(n3.clone(), n6.clone());
    Inode::push(n5.clone(), n7.clone());
    Inode::push(n7.clone(), n8.clone());
    Inode::push(n7.clone(), n9.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    let n3 = n3.lock();
    let n4 = n4.lock();
    let n5 = n5.lock();
    let n6 = n6.lock();
    let n7 = n7.lock();
    let n8 = n8.lock();
    let n9 = n9.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());
    info!("n7:{:?}", n7.borrow());
    info!("n8:{:?}", n8.borrow());
    info!("n9:{:?}", n9.borrow());
  }

  #[test]
  fn get2() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = Tvalue { value: 1 };
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(None, v1, s1);
    let nid1 = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = Tvalue { value: 2 };
    let s2 = IRect::new((0, 0), (20, 20));
    let us2 = U16Rect::new((0, 0), (20, 20));
    let n2 = Tnode::new(None, v2, s2);
    let nid2 = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = Tvalue { value: 3 };
    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = Tnode::new(None, v3, s3);
    let nid3 = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = Tvalue { value: 4 };
    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = Tnode::new(None, v4, s4);
    let nid4 = n4.id();
    let n4 = Tnode::to_arc(n4);

    let v5 = Tvalue { value: 5 };
    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((0, 0), (15, 20));
    let n5 = Tnode::new(None, v5, s5);
    let nid5 = n5.id();
    let n5 = Tnode::to_arc(n5);

    let v6 = Tvalue { value: 5 };
    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((8, 13), (15, 20));
    let n6 = Tnode::new(None, v6, s6);
    let nid6 = n6.id();
    let n6 = Tnode::to_arc(n6);

    /**
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
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n4.clone(), n5.clone());
    Inode::push(n5.clone(), n6.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    let n3 = n3.lock();
    let n4 = n4.lock();
    let n5 = n5.lock();
    let n6 = n6.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());

    let expects = vec![us1, us2, us3, us4, us5, us6];
    let nodes = vec![n1, n2, n3, n4, n5, n6];
    for i in 0..9 {
      let expect = expects[i];
      let node = &nodes[i];
      let actual = node.borrow().actual_shape();
      assert_eq!(expect, actual);
    }
  }
}
