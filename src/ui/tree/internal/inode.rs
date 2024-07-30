//! Internal tree structure implementation: the `Inode` structure.

use std::collections::{BTreeMap, VecDeque};
use std::ops::FnMut;
use std::sync::{Arc, RwLock, Weak};

use crate::cart::{shapes, IRect, U16Rect};
use crate::uuid;

#[derive(Debug, Clone)]
pub struct Inode<T> {
  parent: Option<InodeWk<T>>,
  /// The children collection is sorted by the z-index.
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

  pub fn ptr(node: Inode<T>) -> InodePtr<T> {
    Arc::new(RwLock::new(node))
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

  pub fn set_parent(&mut self, parent: Option<InodeWk<T>>) -> Option<InodeWk<T>> {
    let old_parent = self.parent;
    self.parent = parent;
    old_parent
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
    Inode::level_order_traverse(start_node, start_parent_node, |start, parent| {
      let read_start = start.write().unwrap();
      let read_parent = parent.read().unwrap();
      read_start.attr.depth = read_parent.attr.depth + 1;
    });
  }

  /// Calculate and update all descendants actual shapes, start from the `start_node`.
  fn update_actual_shape(start_node: InodePtr<T>, start_parent_node: InodePtr<T>) {
    Inode::level_order_traverse(start_node, start_parent_node, |start, parent| {
      let read_start = start.write().unwrap();
      let read_parent = parent.read().unwrap();
      read_start.attr.actual_shape =
        shapes::convert_to_actual_shape(read_start.attr.shape, read_parent.attr.actual_shape);
    });
  }

  /// Level-order traverse the sub-tree, start from `start_node`, and apply the `f` function on
  /// each node with its parent.
  fn level_order_traverse(
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
            que.push_back((child, c));
          }
        }
        None => { /* Do nothing */ }
      }
    }
  }

  /// Push a child node into the children vector.
  /// This operation also sorts the newly inserted node with other children by the z-index. It also
  /// calculates and updates the attributes for the pushed node and all its descendants.
  ///
  /// Note: You need to manually assigned the `parent` pointer inside the `child` node, outside of
  /// this method.
  /// Because this (`self`) node doesn't have its `std::sync::Arc` pointer, so this method cannot
  /// do this for you.
  pub fn push(&mut self, child: InodePtr<T>) {
    if self.children.is_none() {
      self.children = Some(BTreeMap::new());
    }
    self.update_attribute(child, self);
    self
      .children
      .unwrap()
      .insert(child.read().unwrap().attr.zindex, child)
  }

  /// Pop a child node from the end of the children vector.
  /// This operation also removes the connection between this (`self`) node and the removed child.
  pub fn pop(&mut self) -> Option<InodePtr<T>> {
    if let Some(&mut children) = self.children {
      if let Some((_, child)) = children.pop_first() {
        child.write().unwrap().parent = None;
        return Some(child);
      }
    }
    None
  }

  /// Remove a child node by its ID from the children vector.
  /// This operation also removes the connection between this (`self`) node and the removed child.
  pub fn remove(&mut self, id: usize) -> Option<InodePtr<T>> {
    if let Some(&mut children) = self.children {
      for (_, child) in children.iter() {}
      if let Some((_, child)) = children.pop_first() {
        child.write().unwrap().parent = None;
        return Some(child);
      }
    }
    None
  }

  /// Get descendant child by its ID, i.e. search in all children nodes in the sub-tree.
  pub fn get_descendant_child(&self, id: usize) -> Option<InodePtr<T>> {
    let mut q: VecDeque<InodePtr<T>> = match self.children {
      Some(children) => children.iter().map(|(_, c)| c).collect(),
      None => vec![].iter().collect(),
    };
    while let Some(e) = q.pop_front() {
      if e.read().unwrap().id() == id {
        return Some(e);
      }
      match e.children {
        Some(children) => {
          for (_, c) in children.iter() {
            q.push_back(c);
          }
        }
        None => { /* Do nothing */ }
      }
    }
    None
  }

  // Children }
}
