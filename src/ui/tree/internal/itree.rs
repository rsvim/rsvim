//! Internal tree structure implementation: the `Itree` structure.

use std::sync::Arc;
use std::{collections::VecDeque, iter::Iterator};

use crate::ui::tree::internal::inode::{Inode, InodeArc, InodeValue};

#[derive(Debug, Clone, Default)]
pub struct Itree<T>
where
  T: InodeValue,
{
  root: Option<InodeArc<T>>,
}

#[derive(Debug, Clone)]
/// The pre-order iterator of the tree.
///
/// It iterates the tree nodes following the order of rendering, i.e. the nodes with lower z-index
/// that can be covered by other nodes are visited earlier, the nodes with higher z-index that will
/// cover other nodes are visited later.
pub struct ItreeIterator<T>
where
  T: InodeValue,
{
  order: ItreeIterateOrder,
  queue: VecDeque<InodeArc<T>>,
}

impl<T> Iterator for ItreeIterator<T>
where
  T: InodeValue,
{
  type Item = InodeArc<T>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(node) = self.queue.pop_front() {
      match self.order {
        ItreeIterateOrder::Ascent => {
          for child in node.lock().borrow().children().iter() {
            self.queue.push_back(child.clone());
          }
        }
        ItreeIterateOrder::Descent => {
          for child in node.lock().borrow().children().iter().rev() {
            self.queue.push_back(child.clone());
          }
        }
      }
      return Some(node);
    }
    None
  }
}

impl<T> ItreeIterator<T>
where
  T: InodeValue,
{
  pub fn new(start: Option<InodeArc<T>>, order: ItreeIterateOrder) -> Self {
    let mut queue = VecDeque::new();
    match start {
      Some(start_node) => queue.push_back(start_node),
      None => { /* Do nothing */ }
    }
    ItreeIterator { order, queue }
  }
}

#[derive(Debug, Clone)]
/// The iterator's visiting order for all children nodes under the same node.
///
/// The `Ascent` visits from lower z-index to higher.
/// The `Descent` visits from higher z-index to lower.
pub enum ItreeIterateOrder {
  Ascent,
  Descent,
}

impl<T> Itree<T>
where
  T: InodeValue,
{
  pub fn new() -> Self {
    Itree { root: None }
  }

  pub fn is_empty(&self) -> bool {
    self.root.is_none()
  }

  pub fn root(&self) -> Option<InodeArc<T>> {
    self.root.as_ref().map(|root| root.clone())
  }

  pub fn set_root(&mut self, root: Option<InodeArc<T>>) -> Option<InodeArc<T>> {
    let old = self.root.as_ref().map(|root| root.clone());
    self.root = root;
    old
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
  /// By default it iterates in pre-order, start from the root. For the children under the same
  /// node, it visits from lower z-index to higher.
  pub fn iter(&self) -> ItreeIterator<T> {
    ItreeIterator::new(self.root.clone(), ItreeIterateOrder::Ascent)
  }

  /// Get the iterator with specified order.
  pub fn ordered_iter(&self, order: ItreeIterateOrder) -> ItreeIterator<T> {
    ItreeIterator::new(self.root.clone(), order)
  }

  /// Insert a child node into the parent node.
  ///
  /// Note:
  /// 1. When no parent node is provided, the node is inserted as the root node of the tree.
  /// 2. When a parent node is provided, the node is inserted as the child node of the parent node,
  ///    you need to get the parent node before insert.
  pub fn insert(&mut self, parent: Option<InodeArc<T>>, child: InodeArc<T>) -> Option<InodeArc<T>> {
    match parent {
      Some(parent) => {
        self.assert_exists(parent.clone());
        child
          .write()
          .unwrap()
          .set_parent(Some(Arc::downgrade(&parent)));
        Inode::push(parent, child.clone());
        Some(child.clone())
      }
      None => {
        assert!(
          self.root.is_none(),
          "Root node already exists when inserting without parent"
        );
        self.root = Some(child.clone());
        Some(child.clone())
      }
    }
  }

  /// Get a node by its ID.
  pub fn get(&self, id: usize) -> Option<InodeArc<T>> {
    match self.root.clone() {
      Some(root) => root.read().unwrap().get_descendant(id),
      None => None,
    }
  }

  /// Remove a node from the parent node.
  pub fn remove(parent: InodeArc<T>, index: usize) -> Option<InodeArc<T>> {
    parent.write().unwrap().remove(index)
  }
}
