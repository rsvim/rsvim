//! Internal tree structure implementation: the `Inode` structure.

use std::collections::VecDeque;
use std::ops::FnMut;
use std::sync::{Arc, RwLock, Weak};

use crate::cart::{shapes, IRect, U16Rect};
use crate::uuid;

#[derive(Debug, Clone)]
pub struct Inode<T> {
  /// Parent.
  parent: Option<InodeWk<T>>,

  /// The children collection is ascent sorted by the z-index, i.e. from lower to higher.
  children: Option<Vec<InodeArc<T>>>,

  /// Attributes
  value: T,
  id: usize,
  depth: usize,
  shape: IRect,
  actual_shape: U16Rect,
  zindex: usize,
  enabled: bool,
  visible: bool,
}

pub type InodeArc<T> = Arc<RwLock<Inode<T>>>;
pub type InodeWk<T> = Weak<RwLock<Inode<T>>>;

impl<T> Inode<T> {
  pub fn new(parent: Option<InodeWk<T>>, value: T, shape: IRect) -> Self {
    Inode {
      parent,
      children: None,
      value,
      id: uuid::next(),
      depth: 0,
      shape,
      actual_shape: U16Rect::new((0, 0), (0, 0)),
      zindex: 0,
      enabled: true,
      visible: true,
    }
  }

  pub fn arc(node: Inode<T>) -> InodeArc<T> {
    Arc::new(RwLock::new(node))
  }

  // Attribute {

  pub fn id(&self) -> usize {
    self.id
  }

  pub fn depth(&self) -> usize {
    self.depth
  }

  pub fn shape(&self) -> IRect {
    self.shape
  }

  pub fn actual_shape(&self) -> U16Rect {
    self.actual_shape
  }

  pub fn zindex(&self) -> usize {
    self.zindex
  }

  pub fn enabled(&self) -> bool {
    self.enabled
  }

  pub fn visible(&self) -> bool {
    self.visible
  }

  pub fn value(&self) -> &T {
    &self.value
  }

  // Attribute }

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

  pub fn children(&self) -> Option<&Vec<InodeArc<T>>> {
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
  fn update_attribute(start_node: InodeArc<T>, start_parent_node: InodeArc<T>) {
    Inode::level_order_traverse(start_node, start_parent_node, |start, parent| {
      let parent2 = parent.read().unwrap();
      let mut start2 = start.write().unwrap();
      start2.attr.depth = parent2.attr.depth + 1;
    });
    Inode::level_order_traverse(start_node, start_parent_node, |start, parent| {
      let parent2 = parent.read().unwrap();
      let mut start2 = start.write().unwrap();
      start2.attr.actual_shape =
        shapes::convert_to_actual_shape(start2.attr.shape, parent2.attr.actual_shape);
    });
  }

  /// Level-order traverse the sub-tree, start from `start_node`, and apply the `f` function on
  /// each node with its parent.
  fn level_order_traverse(
    start_node: InodeArc<T>,
    start_parent_node: InodeArc<T>,
    mut f: dyn FnMut(InodeArc<T>, InodeArc<T>),
  ) {
    f(start_node, start_parent_node);

    let start = start_node.read().unwrap();
    let mut que: VecDeque<(InodeArc<T>, InodeArc<T>)> = match start.children {
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
  /// Note: You need to manually assigned the `parent` pointer inside the `child` node to this
  /// (`self`) node, outside of this method.
  /// Because this (`self`) node doesn't have the related `std::sync::Arc` pointer, so this method
  /// cannot do this for you.
  pub fn push(parent: InodeArc<T>, child: InodeArc<T>) {
    let mut start_node = parent.write().unwrap();
    if start_node.children.is_none() {
      start_node.children = Some(Vec::new());
    }

    // Update attributes start from `child`, and all its descendants.
    Inode::update_attribute(child, parent);

    // Insert `child` by the order of z-index.
    let child_zindex = child.read().unwrap().attr.zindex;
    let mut higher_zindex_pos: Vec<usize> = parent
      .read()
      .unwrap()
      .children
      .unwrap()
      .iter()
      .enumerate()
      .filter(|(index, c)| c.read().unwrap().attr.zindex >= child_zindex)
      .map(|(index, c)| index)
      .rev()
      .collect();
    match higher_zindex_pos.pop() {
      Some(insert_pos) => {
        // Got the first child's position that has higher z-index, insert before it.
        parent
          .write()
          .unwrap()
          .children
          .unwrap()
          .insert(insert_pos, child)
      }
      None => {
        // No existed children has higher z-index, insert at the end.
        parent.write().unwrap().children.unwrap().push(child)
      }
    }
  }

  pub fn first(&self) -> Option<InodeArc<T>> {
    match self.children {
      Some(children) => {
        if children.is_empty() {
          None
        } else {
          Some(children[0])
        }
      }
      None => None,
    }
  }

  pub fn last(&self) -> Option<InodeArc<T>> {
    match self.children {
      Some(children) => {
        if children.is_empty() {
          None
        } else {
          Some(children[children.len() - 1])
        }
      }
      None => None,
    }
  }

  /// Pop a child node from the end of the children vector.
  /// This operation also removes the connection between this (`self`) node and the removed child.
  pub fn pop(&mut self) -> Option<InodeArc<T>> {
    if let Some(&mut children) = self.children {
      if let Some(child) = children.pop() {
        child.write().unwrap().parent = None;
        return Some(child);
      }
    }
    None
  }

  /// Remove a child node by index from the children vector.
  /// This operation also removes the connection between this (`self`) node and the removed child.
  pub fn remove(&mut self, index: usize) -> Option<InodeArc<T>> {
    if let Some(&mut children) = self.children {
      if children.len() > index {
        let removed_child = children.remove(index);
        Some(removed_child)
      } else {
        None
      }
    }
    None
  }

  /// Get descendant child by its ID, i.e. search in all children nodes in the sub-tree.
  pub fn get_descendant(&self, id: usize) -> Option<InodeArc<T>> {
    let mut q: VecDeque<InodeArc<T>> = match self.children {
      Some(children) => children.iter().collect(),
      None => vec![].iter().collect(),
    };
    while let Some(e) = q.pop_front() {
      if e.read().unwrap().id() == id {
        return Some(e);
      }
      match e.children {
        Some(children) => {
          for child in children.iter() {
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
