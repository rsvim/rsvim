//! Internal tree structure implementation: the `Inode` structure.

use std::collections::VecDeque;
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
  pub shape: IRect,
  pub actual_shape: U16Rect,
  pub zindex: usize,
  pub enabled: bool,
  pub visible: bool,
}

impl InodeAttr {
  pub fn new(shape: IRect, actual_shape: U16Rect) -> Self {
    InodeAttr {
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

  fn update_actual_shape(start: InodePtr<T>, start_parent: InodePtr<T>) {
    let parent_actual_shape = start_parent.read().unwrap().attr.actual_shape;
    let mut start1 = start.write().unwrap();
    start1.attr.actual_shape =
      shapes::convert_to_actual_shape(start1.attr.shape, parent_actual_shape);

    let mut q: VecDeque<(InodePtr<T>, InodePtr<T>)> = match start1.children {
      Some(children) => children.iter().map(|child| (start1, child)).collect(),
      None => vec![].iter().collect(),
    };

    while let Some(parent_child_pair) = q.pop_front() {
      let parent_actual_shape = parent_child_pair.0.read().unwrap().attr.actual_shape;
      let mut child2 = parent_child_pair.1.write().unwrap();
      let shape = child2.attr.shape;
      child2.attr.actual_shape = shapes::convert_to_actual_shape(shape, parent_actual_shape);
      match child2.children {
        Some(children) => {
          for c in children.iter() {
            q.push_back(c);
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
    self.children.unwrap().push(child);

    let mut child1 = child.write().unwrap();
    child1.attr.actual_shape =
      shapes::convert_to_actual_shape(child1.attr.shape, self.attr.actual_shape);

    let mut q: VecDeque<(InodePtr<T>, InodePtr<T>)> = match child1.children {
      Some(children) => children.iter().map(|child| (child1, child)).collect(),
      None => vec![].iter().collect(),
    };
    while let Some(parent_child_pair) = q.pop_front() {
      let parent2 = parent_child_pair.0;
      let child2 = parent_child_pair.1;
      let shape = child2.read().unwrap().attr.shape;
      let parent_actual_shape = parent2.read().unwrap().attr.actual_shape;
      child2.write().unwrap().attr.actual_shape =
        shapes::convert_to_actual_shape(shape, parent_actual_shape);
      match child2.read().unwrap().children {
        Some(children) => {
          for c in children.iter() {
            q.push_back(c);
          }
        }
        None => { /* Do nothing */ }
      }
    }
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
