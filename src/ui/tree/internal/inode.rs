//! Internal tree structure implementation: the `Inode` structure.

use std::collections::VecDeque;
use std::ops::FnMut;
use std::sync::{Arc, RwLock, Weak};

use crate::cart::{shapes, IRect, U16Rect};
use crate::uuid;

#[derive(Debug, Clone)]
pub struct Inode<T> {
  parent: Option<InodeWk<T>>,
  children: Option<Vec<InodePtr<T>>>,
  id: usize,
  value: T,
  attr: InodeAttr,
}

pub type InodePtr<T> = Arc<RwLock<Inode<T>>>;
pub type InodeWk<T> = Weak<RwLock<Inode<T>>>;

#[derive(Debug, Clone, Copy)]
pub struct InodeAttr {
  pub depth: usize,
  pub shape: IRect,
  pub actual_shape: U16Rect,
  pub zindex: usize,
  pub enabled: bool,
  pub visible: bool,
}

impl InodeAttr {
  pub fn new(depth: usize, shape: IRect, actual_shape: U16Rect) -> Self {
    InodeAttr {
      depth,
      shape,
      actual_shape,
      zindex: 0,
      enabled: true,
      visible: true,
    }
  }
}

impl<T> Inode<T> {
  pub fn new(parent: Option<InodeWk<T>>, value: T, attr: InodeAttr) -> Self {
    Inode {
      parent,
      children: None,
      id: uuid::next(),
      value,
      attr,
    }
  }

  pub fn id(&self) -> usize {
    self.id
  }

  pub fn attribute(&self) -> InodeAttr {
    self.attr
  }

  pub fn value(&self) -> &T {
    &self.value
  }

  // Parent {

  pub fn parent(&self) -> Option<InodeWk<T>> {
    self.parent
  }

  // Parent }

  // Children {

  pub fn children(&self) -> Option<&Vec<InodePtr<T>>> {
    self.children
  }

  /// Calculate and update the `start_node` attributes, based on its parent's attributes.
  /// Also recursively calculate and update all descendants in the sub-tree, start from the
  /// `start_node`.
  ///
  /// These attributes are relative to the parent node, and need to be calculated and updated when
  /// the node is been moved in the tree:
  ///
  /// 1. [`depth`](InodeAttr::depth)
  /// 2. [`actual_shape`](InodeAttr::actual_shape)
  fn update_attribute(start_node: InodePtr<T>, start_parent_node: InodePtr<T>) {
    Inode::update_depth(start_node, start_parent_node);
    Inode::update_actual_shape(start_node, start_parent_node);
  }

  /// Calculate and update all descendants depths, start from the `start_node`.
  fn update_depth(start_node: InodePtr<T>, start_parent_node: InodePtr<T>) {
    Inode::level_order_traversal(start_node, start_parent_node, |start, parent| {
      let start1 = start.write().unwrap();
      let parent1 = parent.read().unwrap();
      start1.attr.depth = parent1.attr.depth + 1;
    });
  }

  /// Calculate and update all descendants actual shapes, start from the `start_node`.
  fn update_actual_shape(start_node: InodePtr<T>, start_parent_node: InodePtr<T>) {
    Inode::level_order_traversal(start_node, start_parent_node, |start, parent| {
      let start1 = start.write().unwrap();
      let parent1 = parent.read().unwrap();
      start1.attr.actual_shape =
        shapes::convert_to_actual_shape(start1.attr.shape, parent1.attr.actual_shape);
    });
  }

  fn level_order_traversal(
    start_node: InodePtr<T>,
    start_parent_node: InodePtr<T>,
    mut f: dyn FnMut(InodePtr<T>, InodePtr<T>),
  ) {
    f(start_node, start_parent_node);

    let start = start_node.read().unwrap();
    let mut que: VecDeque<(InodePtr<T>, InodePtr<T>)> = match start.children {
      Some(children) => children.iter().map(|c| (start, c)).collect(),
      None => vec![].iter().collect(),
    };

    while let Some(parent_child_pair) = que.pop_front() {
      let parent = parent_child_pair.0;
      let child = parent_child_pair.1;
      f(child, parent);
      let child = child.read().unwrap();
      match child.children {
        Some(children) => {
          for c in children.iter() {
            que.push_back(c);
          }
        }
        None => { /* Do nothing */ }
      }
    }
  }

  /// Push a child node at the end of children's vector.
  /// This operation also calculates and updates the actual shape for the pushed node and all its
  /// descendant children.
  pub fn push(&mut self, child: InodePtr<T>) {
    if self.children.is_none() {
      self.children = Some(vec![]);
    }
    self.update_actual_shape(child, self.attr.actual_shape);
    self.children.unwrap().push(child)
  }

  pub fn insert(&mut self, index: usize, child: InodePtr<T>) {
    if self.children.is_none() {
      self.children = Some(vec![]);
    }
    self.update_actual_shape(child, self.attr.actual_shape);
    self.children.unwrap().insert(index, child)
  }

  /// Pop a child node from the end of the chlidren's vector.
  pub fn pop(&mut self) -> Option<InodePtr<T>> {
    match self.children {
      Some(&mut children) => children.pop(),
      None => None,
    }
  }

  /// Remove a child node.
  pub fn remove(&mut self, index: usize) -> Option<InodePtr<T>> {
    match self.children {
      Some(&mut children) => Some(children.remove(index)),
      None => None,
    }
  }

  /// Get descendant child by its ID, i.e. search in all children nodes in the sub-tree.
  pub fn get_descendant_child(&self, id: usize) -> Option<InodePtr<T>> {
    let mut q: VecDeque<InodePtr<T>> = match self.children {
      Some(c) => c.iter().collect(),
      None => vec![].iter().collect(),
    };
    while let Some(e) = q.pop_front() {
      if e.read().unwrap().id() == id {
        return Some(e);
      }
      match e.children {
        Some(ec) => {
          for child in ec.iter() {
            q.push_back(child);
          }
        }
        None => { /* Do nothing */ }
      }
    }
    None
  }

  // Children }
}
