//! Internal tree structure implementation: the `Itree` structure.

use std::{collections::VecDeque, iter::Iterator};

use crate::ui::tree::internal::inode::InodePtr;

#[derive(Debug, Clone)]
pub struct Itree<T> {
  root: Option<InodePtr<T>>,

  /// As the widget tree, there's a focus node, i.e. the current widget that the position of the
  /// user's cursor.
  current: Option<InodePtr<T>>,
}

#[derive(Debug, Clone)]
/// The pre-order iterator of the tree.
///
/// It iterates the tree nodes following the order of rendering, i.e. the nodes with lower z-index
/// that can be covered by other nodes are visited earlier, the nodes with higher z-index that will
/// cover other nodes are visited later.
pub struct ItreeIterator<T> {
  /// All children under the same node is iterated by the order of z-index value, either ascent or
  /// descent, i.e. from low to high or high to low.
  order: ItreeIterateOrder,
  queue: VecDeque<InodePtr<T>>,
}

impl<T> Iterator for ItreeIterator<T> {
  type Item = InodePtr<T>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(node) = self.queue.pop_front() {
      match node.read().unwrap().children() {
        Some(children) => match self.order {
          ItreeIterateOrder::Ascent => {
            for (zindex, child) in children.iter() {
              self.queue.push_back(child);
            }
          }
          ItreeIterateOrder::Descent => {
            for (zindex, child) in children.iter().rev() {
              self.queue.push_back(child);
            }
          }
        },
        None => { /* Do nothing */ }
      }
      Some(node)
    }
    None
  }
}

impl<T> ItreeIterator<T> {
  pub fn new(start: Option<InodePtr<T>>, order: ItreeIterateOrder) -> Self {
    let mut q = VecDeque::new();
    match start {
      Some(start_node) => q.push_back(start_node),
      None => { /* Do nothing */ }
    }
    ItreeIterator {
      order: ItreeIterateOrder::Ascent,
      queue: q,
    }
  }
}

pub enum ItreeIterateOrder {
  // Iterate by z-index value, from smallest to biggest.
  Ascent,
  // Iterate by z-index value, from biggest to smallest.
  Descent,
}

impl<T> Itree<T> {
  pub fn new() -> Self {
    Itree {
      root: None,
      current: None,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.root.is_none()
  }

  pub fn root(&self) -> Option<InodePtr<T>> {
    self.root
  }

  pub fn current(&self) -> Option<InodePtr<T>> {
    self.current
  }

  /// Assert the `node` exists in the tree.
  ///
  /// # Panics
  ///
  /// Panics when the `node` doesn't exist.
  fn assert_exists(&self, node: InodePtr<T>) {
    assert!(
      self.root.is_some(),
      "Doesn't have a root node when assert the node exists"
    );
    let node = node.write().unwrap();
    let node2 = self
      .root
      .unwrap()
      .write()
      .unwrap()
      .get_descendant_child(node.id());
    assert!(node2.is_some(), "Missing node {} in the tree", node.id());
    assert!(
      node2.unwrap().read().unwrap().id() == node.id(),
      "Node ID {} not match in the tree",
      node.id()
    );
  }

  /// Assert the `node` is the root node.
  ///
  /// # Panics
  ///
  /// Panics if the `node` isn't the root node.
  fn assert_is_root(&self, node: InodePtr<T>) {}

  /// Assert the `node` is not the root node, but exists in the tree.
  ///
  /// # Panics
  ///
  /// Panics if the `node` is the root node.
  fn assert_not_root(&self, node: InodePtr<T>) {}

  pub fn set_current(&mut self, node: InodePtr<T>) {
    match self.root {
      Some(root) => {}
      None => {}
    }
  }

  /// Get the iterator.
  ///
  /// By default the iterate order is level order start from the root node, for all the children
  /// under the same node, traverse from smallest z-index to biggest.
  pub fn iter(&self) -> ItreeIterator<T> {
    ItreeIterator::new(self.root, ItreeIterateOrder::Ascent)
  }

  /// Get the iterator in a children-descent-order.
  ///
  /// For all the children under the same node, traverse from highest z-index to smallest.
  pub fn iter_descent(&self) -> ItreeIterator<T> {
    ItreeIterator::new(self.root, ItreeIterateOrder::Descent)
  }

  pub fn insert(&mut self, parent: Option<InodePtr<T>>, node: InodePtr<T>) -> Option<InodePtr<T>> {
    match parent {
      Some(parent) => {
        self.assert_exists(parent);

        node.write().unwrap().set_parent(parent);
        parent.write().unwrap().push(node);
        Some(node)
      }
      None => {
        assert!(
          self.root.is_none(),
          "Root node already exists when inserting without parent"
        );
        self.root = Some(node);
        Some(node)
      }
    }
  }
}
