//! Internal tree structure implementation: the `Itree` structure.

use std::{collections::VecDeque, iter::Iterator};

use crate::ui::tree::internal::inode::{Inode, InodeAttr, InodePtr};

#[derive(Debug, Clone)]
pub struct Itree<T> {
  root: Option<InodePtr<T>>,
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
  pub fn new(start: InodePtr<T>, order: ItreeIterateOrder) -> Self {
    let mut q = VecDeque::new();
    q.push_back(start);
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
  pub fn new(root_value: T, root_attr: InodeAttr) -> Self {
    let node = Inode::new(None, root_value, root_attr);
    Itree {
      root: Some(Inode::ptr(node)),
    }
  }

  pub fn iter(&self, order: ItreeIterateOrder) -> ItreeIterator<T> {
    ItreeIterator::new(self.root, order)
  }
}
