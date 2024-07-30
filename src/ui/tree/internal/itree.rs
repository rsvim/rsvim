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
/// The level-order iterator of the tree.
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
    ItreeIterator { order, queue: q }
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
    Itree { root: None }
  }

  pub fn root(&self) -> Option<InodePtr<T>> {
    self.root
  }

  pub fn iter(&self, order: ItreeIterateOrder) -> ItreeIterator<T> {
    ItreeIterator::new(self.root, order)
  }

  pub fn insert(&mut self, parent: Option<InodePtr<T>>, node: InodePtr<T>) -> Option<InodePtr<T>> {
    match parent {
      Some(parent) => {
        assert!(
          self.root.is_some(),
          "Doesn't have a root node when inserting with parent"
        );
        let write_parent = parent.write().unwrap();
        let get_parent = self
          .root
          .unwrap()
          .write()
          .unwrap()
          .get_descendant_child(write_parent.id());
        assert!(
          get_parent.is_some(),
          "Missing parent {} in the tree",
          write_parent.id()
        );
        assert!(
          get_parent.unwrap().read().unwrap().id() == write_parent.id(),
          "Parent ID {} not match in the tree",
          write_parent.id()
        );

        node.write().unwrap().set_parent(parent);
        write_parent.push(node);
        Some(node)
      }
      None => {
        assert!(
          self.root.is_none(),
          "Root node exists when inserting without parent"
        );
        self.root = Some(node);
        Some(node)
      }
    }
  }

  pub fn current(&self) -> Option<InodePtr<T>> {
    self.current
  }
}
