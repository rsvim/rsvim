//! Internal tree structure implementation: the `Itree` structure.

use std::{collections::VecDeque, iter::Iterator};

use crate::ui::tree::internal::inode::{Inode, InodeAttr, InodePtr};

#[derive(Debug, Clone)]
pub struct Itree<T> {
  root: Option<InodePtr<T>>,
}

#[derive(Debug, Clone)]
pub struct ItreeIterator<T> {
  order: ItreeIterateOrder,
  current: InodePtr<T>,
  queue: VecDeque<InodePtr<T>>,
}

impl<T> Iterator for ItreeIterator<T> {
  type Item = InodePtr<T>;

  fn next(&mut self) -> Option<Self::Item> {
    match self.order {
      ItreeIterateOrder::LevelOrder => self.level_order_next(),
      ItreeIterateOrder::InOrder => self.in_order_next(),
      ItreeIterateOrder::PreOrder => self.pre_order_next(),
      ItreeIterateOrder::PostOrder => self.post_order_next(),
    }
  }
}

impl<T> ItreeIterator<T> {
  pub fn new(current_node: InodePtr<T>, order: ItreeIterateOrder) -> Self {
    ItreeIterator {
      current: current_node,
      order,
    }
  }

  fn level_order_next(&mut self) -> Option<InodePtr<T>> {
    match self.current {
      Some(&mut current_node) => {}
      None => None,
    }
  }
  fn in_order_next(&mut self) -> Option<InodePtr<T>> {}
  fn pre_order_next(&mut self) -> Option<InodePtr<T>> {}
  fn post_order_next(&mut self) -> Option<InodePtr<T>> {}
}

pub enum ItreeIterateOrder {
  LevelOrder,
  InOrder,
  PreOrder,
  PostOrder,
}

impl<T> Itree<T> {
  pub fn new(root_value: T, root_attr: InodeAttr) -> Self {
    let node = Inode::new(None, root_value, root_attr);
    Itree {
      root: Some(Inode::ptr(node)),
    }
  }

  pub fn iter(&self) -> ItreeIterator<T> {}
}
