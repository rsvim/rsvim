//! Internal tree structure implementation: the `Inode` structure.

use geo::point;
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::FnMut;
use std::sync::{Arc, Weak};

use crate::cart::{shapes, IRect, U16Rect};
use crate::geo_rect_as;
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
    let actual_shape = geo_rect_as!(shape, u16);
    Inode {
      parent,
      children: vec![],
      value,
      id: uuid::next(),
      depth: 0,
      shape,
      actual_shape,
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

  /// Calculate and update the `start` attributes, based on its parent's attributes. Also
  /// recursively calculate and update all descendants in the sub-tree, start from the `start`.
  ///
  /// These attributes are relative to the parent node, and need to be calculated and updated when
  /// the node is been moved in the tree:
  ///
  /// 1. [`depth`](InodeAttr::depth)
  /// 2. [`actual_shape`](InodeAttr::actual_shape)
  fn update_attribute(start: InodeArc<T>, parent: InodeArc<T>) {
    let mut update_depth: fn(InodeArc<T>, InodeArc<T>) = |child, parent| {
      let parent = parent.lock();
      let child = child.lock();
      child.borrow_mut().depth = parent.borrow().depth + 1;
    };
    let mut update_actual_shape: fn(InodeArc<T>, InodeArc<T>) = |child, parent| {
      let parent = parent.lock();
      let child = child.lock();
      child.borrow_mut().actual_shape =
        shapes::convert_to_actual_shape(child.borrow().shape, parent.borrow().actual_shape);
    };

    Inode::level_order_traverse(start.clone(), parent.clone(), &mut update_depth);
    Inode::level_order_traverse(start, parent, &mut update_actual_shape);
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
  pub fn push(parent: InodeArc<T>, child: InodeArc<T>) {
    // Update attributes start from `child`, and all its descendants.
    Inode::update_attribute(child.clone(), parent.clone());

    // Assign the `parent` pointer for `child`.
    child.lock().borrow_mut().parent = Some(Arc::downgrade(&parent));

    // Insert `child` into the children vector, by the order of z-index.
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cart::IRect;
  use crate::test::log::init as test_log_init;
  use std::sync::Once;
  use tracing::info;

  #[derive(Clone, Debug)]
  struct TestValue {
    pub value: usize,
  }

  impl InodeValue for TestValue {}

  // Test node
  type Tnode = Inode<TestValue>;

  static INIT: Once = Once::new();

  #[test]
  fn new_node() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = TestValue { value: 1 };
    let s1 = IRect::new((0, 0), (1, 1));
    let us1 = U16Rect::new((0, 0), (1, 1));
    let prev_id = uuid::next();
    let n1 = Tnode::new(None, v1.clone(), s1);
    let n1 = Tnode::to_arc(n1);
    let n1 = n1.lock();
    assert_eq!(prev_id + 1, n1.borrow().id());
    assert_eq!(n1.borrow().shape(), s1);
    assert_eq!(n1.borrow().actual_shape(), us1);
    assert_eq!(n1.borrow().zindex(), 0);
    assert_eq!(n1.borrow().depth(), 0);
    assert!(n1.borrow().enabled());
    assert!(n1.borrow().visible());
    assert_eq!(n1.borrow().value().value, v1.value);
    assert!(n1.borrow().parent().is_none());
  }

  #[test]
  fn insert_node() {
    INIT.call_once(|| {
      test_log_init();
    });

    let prev_id = uuid::next();

    let v1 = TestValue { value: 1 };
    let s1 = IRect::new((0, 0), (1, 1));
    let us1 = U16Rect::new((0, 0), (1, 1));
    let n1 = Tnode::new(None, v1.clone(), s1);
    let n1_id = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = TestValue { value: 1 };
    let s2 = IRect::new((0, 0), (1, 1));
    let us2 = U16Rect::new((0, 0), (1, 1));
    let n2 = Tnode::new(None, v2.clone(), s2);
    let n2_id = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = TestValue { value: 1 };
    let s3 = IRect::new((0, 0), (1, 1));
    let us3 = U16Rect::new((0, 0), (1, 1));
    let n3 = Tnode::new(None, v3.clone(), s3);
    let n3_id = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = TestValue { value: 1 };
    let s4 = IRect::new((0, 0), (1, 1));
    let us4 = U16Rect::new((0, 0), (1, 1));
    let n4 = Tnode::new(None, v4.clone(), s4);
    let n4_id = n4.id();
    let n4 = Tnode::to_arc(n4);

    let v5 = TestValue { value: 1 };
    let s5 = IRect::new((0, 0), (1, 1));
    let us5 = U16Rect::new((0, 0), (1, 1));
    let n5 = Tnode::new(None, v5.clone(), s5);
    let n5_id = n5.id();
    let n5 = Tnode::to_arc(n5);

    let v6 = TestValue { value: 1 };
    let s6 = IRect::new((0, 0), (1, 1));
    let us6 = U16Rect::new((0, 0), (1, 1));
    let n6 = Tnode::new(None, v6.clone(), s6);
    let n6_id = n6.id();
    let n6 = Tnode::to_arc(n6);

    /**
     * The tree looks like:
     *
     *       n1
     *     /   \
     *    n2   n3
     *  /  \     \
     * n4  n5    n6
     *
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n2.clone(), n5.clone());
    Inode::push(n3.clone(), n6.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    assert_eq!(prev_id + 1, n1_id);
    assert_eq!(prev_id + 2, n2_id);
    assert_eq!(n1.borrow().depth() + 1, n2.borrow().depth());
    assert_eq!(n1.borrow().children().len(), 1);
    assert!(
      n1.borrow()
        .children()
        .iter()
        .filter(|c| c.lock().borrow().id() == n2_id)
        .collect::<Vec<_>>()
        .len()
        == 1
    );
  }
}
