//! Internal tree structure implementation: the `Inode` structure.

use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::FnMut;
use std::sync::{Arc, RwLock, Weak};

use crate::cart::{shapes, IRect, U16Rect};
use crate::uuid;

pub trait InodeValue: Sized + Clone + Debug {}

#[derive(Debug, Clone)]
pub struct Inode<T>
where
  T: InodeValue,
{
  /// Parent.
  parent: Option<InodeWk<T>>,

  /// The children collection is ascent sorted by the z-index, i.e. from lower to higher.
  children: Vec<InodeArc<T>>,

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

pub type InodeArc<T> = Arc<ReentrantMutex<RefCell<Inode<T>>>>;
pub type InodeWk<T> = Weak<ReentrantMutex<RefCell<Inode<T>>>>;

impl<T> Inode<T>
where
  T: InodeValue,
{
  pub fn new(parent: Option<InodeWk<T>>, value: T, shape: IRect) -> Self {
    Inode {
      parent,
      children: vec![],
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

  pub fn to_arc(node: Inode<T>) -> InodeArc<T> {
    Arc::new(ReentrantMutex::new(RefCell::new(node)))
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

  pub fn value_mut(&mut self) -> &mut T {
    &mut self.value
  }

  // Attribute }

  // Parent {

  pub fn parent(&self) -> Option<InodeWk<T>> {
    self.parent.clone()
  }

  pub fn set_parent(&mut self, parent: Option<InodeWk<T>>) -> Option<InodeWk<T>> {
    let old_parent = self.parent.clone();
    self.parent = parent;
    old_parent
  }

  // Parent }

  // Children {

  pub fn children(&self) -> &Vec<InodeArc<T>> {
    &self.children
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
    Inode::level_order_traverse(
      start_node.clone(),
      start_parent_node.clone(),
      &mut Inode::update_depth,
    );
    Inode::level_order_traverse(
      start_node,
      start_parent_node,
      &mut Inode::update_actual_shape,
    );
  }

  fn update_depth(child: InodeArc<T>, parent: InodeArc<T>) {
    let parent = parent.lock();
    let child = child.lock();
    child.borrow_mut().depth = parent.borrow().depth + 1;
  }

  fn update_actual_shape(child: InodeArc<T>, parent: InodeArc<T>) {
    let parent = parent.lock();
    let child = child.lock();
    child.borrow_mut().actual_shape =
      shapes::convert_to_actual_shape(child.borrow().shape, parent.borrow().actual_shape);
  }

  /// Level-order traverse the sub-tree, start from `start_node`, and apply the `f` function on
  /// each node with its parent.
  fn level_order_traverse(
    start: InodeArc<T>,
    parent: InodeArc<T>,
    f: &mut dyn FnMut(InodeArc<T>, InodeArc<T>),
  ) {
    f(start.clone(), parent.clone());

    let mut que: VecDeque<(InodeArc<T>, InodeArc<T>)> = VecDeque::from(
      start
        .lock()
        .borrow()
        .children
        .iter()
        .map(|c| (start.clone(), c.clone()))
        .collect::<Vec<(InodeArc<T>, InodeArc<T>)>>(),
    );

    while let Some(parent_child_pair) = que.pop_front() {
      let parent = parent_child_pair.0;
      let child = parent_child_pair.1;
      f(child.clone(), parent.clone());
      for c in child.lock().borrow().children.iter() {
        que.push_back((child.clone(), c.clone()));
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
    // Update attributes start from `child`, and all its descendants.
    Inode::update_attribute(child.clone(), parent.clone());

    // Insert `child` by the order of z-index.
    let child_zindex = child.lock().borrow().zindex;
    let higher_zindex_pos: Vec<usize> = parent
      .lock()
      .borrow()
      .children
      .iter()
      .enumerate()
      .filter(|(_index, c)| c.lock().borrow().zindex > child_zindex)
      .map(|(index, _c)| index)
      .rev()
      .collect();
    match higher_zindex_pos.first() {
      Some(insert_pos) => {
        // Got the first child's position that has higher z-index, insert before it.
        parent
          .lock()
          .borrow_mut()
          .children
          .insert(*insert_pos, child.clone())
      }
      None => {
        // No existed children has higher z-index, insert at the end.
        parent.lock().borrow_mut().children.push(child.clone())
      }
    }
  }

  pub fn first(&self) -> Option<InodeArc<T>> {
    if self.children.is_empty() {
      None
    } else {
      Some(self.children[0].clone())
    }
  }

  pub fn last(&self) -> Option<InodeArc<T>> {
    if self.children.is_empty() {
      None
    } else {
      Some(self.children[self.children.len() - 1].clone())
    }
  }

  /// Pop a child node from the end of the children vector.
  /// This operation also removes the connection between this (`self`) node and the removed child.
  pub fn pop(&mut self) -> Option<InodeArc<T>> {
    match self.children.pop() {
      Some(removed_child) => {
        removed_child.lock().borrow_mut().parent = None;
        Some(removed_child)
      }
      None => None,
    }
  }

  /// Remove a child node by index from the children vector.
  /// This operation also removes the connection between this (`self`) node and the removed child.
  pub fn remove(&mut self, index: usize) -> Option<InodeArc<T>> {
    if self.children.len() > index {
      let removed_child = self.children.remove(index);
      removed_child.lock().borrow_mut().parent = None;
      Some(removed_child)
    } else {
      None
    }
  }

  /// Get descendant child by its ID, i.e. search in all children nodes in the sub-tree.
  pub fn get_descendant(&self, id: usize) -> Option<InodeArc<T>> {
    let mut q: VecDeque<InodeArc<T>> = VecDeque::from(self.children.clone());

    while let Some(e) = q.pop_front() {
      if e.lock().borrow().id() == id {
        return Some(e);
      }
      for child in e.lock().borrow().children.iter() {
        q.push_back(child.clone());
      }
    }
    None
  }

  // Children }
}
