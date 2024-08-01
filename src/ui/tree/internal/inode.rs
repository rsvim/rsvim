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
  use parking_lot::ReentrantMutexGuard;
  use std::sync::Once;
  use tracing::info;

  #[derive(Copy, Clone, Debug)]
  struct TestValue {
    pub value: usize,
  }

  impl InodeValue for TestValue {}

  // Test node
  type Tnode = Inode<TestValue>;

  static INIT: Once = Once::new();

  #[test]
  fn new() {
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
  fn insert1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = TestValue { value: 1 };
    let s1 = IRect::new((0, 0), (1, 1));
    let us1 = U16Rect::new((0, 0), (1, 1));
    let n1 = Tnode::new(None, v1.clone(), s1);
    let n1_id = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = TestValue { value: 2 };
    let s2 = IRect::new((0, 0), (1, 1));
    let us2 = U16Rect::new((0, 0), (1, 1));
    let n2 = Tnode::new(None, v2.clone(), s2);
    let n2_id = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = TestValue { value: 3 };
    let s3 = IRect::new((0, 0), (1, 1));
    let us3 = U16Rect::new((0, 0), (1, 1));
    let n3 = Tnode::new(None, v3.clone(), s3);
    let n3_id = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = TestValue { value: 4 };
    let s4 = IRect::new((0, 0), (1, 1));
    let us4 = U16Rect::new((0, 0), (1, 1));
    let n4 = Tnode::new(None, v4.clone(), s4);
    let n4_id = n4.id();
    let n4 = Tnode::to_arc(n4);

    let v5 = TestValue { value: 5 };
    let s5 = IRect::new((0, 0), (1, 1));
    let us5 = U16Rect::new((0, 0), (1, 1));
    let n5 = Tnode::new(None, v5.clone(), s5);
    let n5_id = n5.id();
    let n5 = Tnode::to_arc(n5);

    let v6 = TestValue { value: 6 };
    let s6 = IRect::new((0, 0), (1, 1));
    let us6 = U16Rect::new((0, 0), (1, 1));
    let n6 = Tnode::new(None, v6.clone(), s6);
    let n6_id = n6.id();
    let n6 = Tnode::to_arc(n6);

    /**
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *      /  \     \
     *     n4  n5    n6
     * ```
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n2.clone(), n5.clone());
    Inode::push(n3.clone(), n6.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    let n3 = n3.lock();
    let n4 = n4.lock();
    let n5 = n5.lock();
    let n6 = n6.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());

    assert_eq!(n1_id + 1, n2_id);
    assert_eq!(n2_id + 1, n3_id);
    assert_eq!(n3_id + 1, n4_id);
    assert_eq!(n4_id + 1, n5_id);
    assert_eq!(n5_id + 1, n6_id);

    assert_eq!(n1.borrow().depth() + 1, n2.borrow().depth());
    assert_eq!(n1.borrow().depth() + 1, n3.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n4.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n5.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n6.borrow().depth());
    assert_eq!(n3.borrow().depth() + 1, n6.borrow().depth());

    assert_eq!(n1.borrow().children().len(), 2);
    assert_eq!(n2.borrow().children().len(), 2);
    assert_eq!(n3.borrow().children().len(), 1);
    assert_eq!(n4.borrow().children().len(), 0);
    assert_eq!(n5.borrow().children().len(), 0);
    assert_eq!(n6.borrow().children().len(), 0);

    let contains_node = |parent: &ReentrantMutexGuard<RefCell<Tnode>>, child_id: usize| -> bool {
      parent
        .borrow()
        .children()
        .iter()
        .filter(|c| c.lock().borrow().id() == child_id)
        .collect::<Vec<_>>()
        .len()
        == 1
    };

    assert!(contains_node(&n1, n2_id));
    assert!(contains_node(&n1, n3_id));
    assert!(!contains_node(&n1, n4_id));
    assert!(!contains_node(&n1, n5_id));
    assert!(!contains_node(&n1, n6_id));

    assert!(contains_node(&n2, n4_id));
    assert!(contains_node(&n2, n5_id));
    assert!(!contains_node(&n2, n6_id));

    assert!(contains_node(&n3, n6_id));
    assert!(!contains_node(&n3, n4_id));
    assert!(!contains_node(&n3, n5_id));
  }

  #[test]
  fn insert2() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = TestValue { value: 1 };
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(None, v1, s1);
    let nid1 = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = TestValue { value: 2 };
    let s2 = IRect::new((0, 0), (15, 15));
    let us2 = U16Rect::new((0, 0), (15, 15));
    let n2 = Tnode::new(None, v2, s2);
    let nid2 = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = TestValue { value: 3 };
    let s3 = IRect::new((10, 10), (18, 19));
    let us3 = U16Rect::new((10, 10), (18, 19));
    let n3 = Tnode::new(None, v3, s3);
    let nid3 = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = TestValue { value: 4 };
    let s4 = IRect::new((3, 5), (20, 14));
    let us4 = U16Rect::new((3, 5), (15, 14));
    let n4 = Tnode::new(None, v4, s4);
    let nid4 = n4.id();
    let n4 = Tnode::to_arc(n4);

    let v5 = TestValue { value: 5 };
    let s5 = IRect::new((-3, -5), (10, 20));
    let us5 = U16Rect::new((0, 0), (10, 15));
    let n5 = Tnode::new(None, v5, s5);
    let nid5 = n5.id();
    let n5 = Tnode::to_arc(n5);

    let v6 = TestValue { value: 6 };
    let s6 = IRect::new((3, 6), (6, 10));
    let us6 = U16Rect::new((13, 16), (16, 19));
    let n6 = Tnode::new(None, v6, s6);
    let nid6 = n6.id();
    let n6 = Tnode::to_arc(n6);

    let v7 = TestValue { value: 7 };
    let s7 = IRect::new((3, 6), (15, 25));
    let us7 = U16Rect::new((3, 6), (10, 15));
    let n7 = Tnode::new(None, v7, s7);
    let nid7 = n7.id();
    let n7 = Tnode::to_arc(n7);

    let v8 = TestValue { value: 8 };
    let s8 = IRect::new((-1, -2), (2, 1));
    let us8 = U16Rect::new((3, 6), (5, 7));
    let n8 = Tnode::new(None, v8, s8);
    let nid8 = n8.id();
    let n8 = Tnode::to_arc(n8);

    let v9 = TestValue { value: 9 };
    let s9 = IRect::new((5, 6), (9, 8));
    let us9 = U16Rect::new((8, 12), (10, 14));
    let n9 = Tnode::new(None, v9, s9);
    let nid9 = n9.id();
    let n9 = Tnode::to_arc(n9);

    /**
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *      /  \     \
     *     n4  n5    n6
     *           \
     *            n7
     *           / \
     *         n8   n9
     * ```
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n2.clone(), n5.clone());
    Inode::push(n3.clone(), n6.clone());
    Inode::push(n5.clone(), n7.clone());
    Inode::push(n7.clone(), n8.clone());
    Inode::push(n7.clone(), n9.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    let n3 = n3.lock();
    let n4 = n4.lock();
    let n5 = n5.lock();
    let n6 = n6.lock();
    let n7 = n7.lock();
    let n8 = n8.lock();
    let n9 = n9.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());
    info!("n7:{:?}", n7.borrow());
    info!("n8:{:?}", n8.borrow());
    info!("n9:{:?}", n9.borrow());

    assert_eq!(nid1 + 1, nid2);
    assert_eq!(nid2 + 1, nid3);
    assert_eq!(nid3 + 1, nid4);
    assert_eq!(nid4 + 1, nid5);
    assert_eq!(nid5 + 1, nid6);
    assert_eq!(nid6 + 1, nid7);
    assert_eq!(nid7 + 1, nid8);
    assert_eq!(nid8 + 1, nid9);

    assert_eq!(n1.borrow().depth() + 1, n2.borrow().depth());
    assert_eq!(n1.borrow().depth() + 1, n3.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n4.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n5.borrow().depth());
    assert_eq!(n2.borrow().depth() + 1, n6.borrow().depth());
    assert_eq!(n3.borrow().depth() + 1, n6.borrow().depth());
    assert_eq!(n5.borrow().depth() + 1, n7.borrow().depth());
    assert_eq!(n7.borrow().depth() + 1, n8.borrow().depth());
    assert_eq!(n7.borrow().depth() + 1, n9.borrow().depth());

    assert_eq!(n1.borrow().children().len(), 2);
    assert_eq!(n2.borrow().children().len(), 2);
    assert_eq!(n3.borrow().children().len(), 1);
    assert_eq!(n4.borrow().children().len(), 0);
    assert_eq!(n5.borrow().children().len(), 1);
    assert_eq!(n6.borrow().children().len(), 0);
    assert_eq!(n7.borrow().children().len(), 2);
    assert_eq!(n8.borrow().children().len(), 0);
    assert_eq!(n9.borrow().children().len(), 0);

    let contains_node = |parent: &ReentrantMutexGuard<RefCell<Tnode>>, child_id: usize| -> bool {
      parent
        .borrow()
        .children()
        .iter()
        .filter(|c| c.lock().borrow().id() == child_id)
        .collect::<Vec<_>>()
        .len()
        == 1
    };

    assert!(contains_node(&n1, nid2));
    assert!(contains_node(&n1, nid3));
    assert!(!contains_node(&n1, nid4));
    assert!(!contains_node(&n1, nid5));
    assert!(!contains_node(&n1, nid7));

    assert!(contains_node(&n2, nid4));
    assert!(contains_node(&n2, nid5));
    assert!(!contains_node(&n2, nid7));

    assert!(contains_node(&n3, nid7));
    assert!(!contains_node(&n3, nid4));
    assert!(!contains_node(&n3, nid5));

    assert!(contains_node(&n5, nid7));
    assert!(contains_node(&n7, nid8));
    assert!(contains_node(&n7, nid9));
  }

  #[test]
  fn shape1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = TestValue { value: 1 };
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(None, v1, s1);
    let nid1 = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = TestValue { value: 2 };
    let s2 = IRect::new((0, 0), (15, 15));
    let us2 = U16Rect::new((0, 0), (15, 15));
    let n2 = Tnode::new(None, v2, s2);
    let nid2 = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = TestValue { value: 3 };
    let s3 = IRect::new((10, 10), (18, 19));
    let us3 = U16Rect::new((10, 10), (18, 19));
    let n3 = Tnode::new(None, v3, s3);
    let nid3 = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = TestValue { value: 4 };
    let s4 = IRect::new((3, 5), (20, 14));
    let us4 = U16Rect::new((3, 5), (15, 14));
    let n4 = Tnode::new(None, v4, s4);
    let nid4 = n4.id();
    let n4 = Tnode::to_arc(n4);

    let v5 = TestValue { value: 5 };
    let s5 = IRect::new((-3, -5), (10, 20));
    let us5 = U16Rect::new((0, 0), (10, 15));
    let n5 = Tnode::new(None, v5, s5);
    let nid5 = n5.id();
    let n5 = Tnode::to_arc(n5);

    let v6 = TestValue { value: 6 };
    let s6 = IRect::new((3, 6), (6, 10));
    let us6 = U16Rect::new((13, 16), (16, 19));
    let n6 = Tnode::new(None, v6, s6);
    let nid6 = n6.id();
    let n6 = Tnode::to_arc(n6);

    let v7 = TestValue { value: 7 };
    let s7 = IRect::new((3, 6), (15, 25));
    let us7 = U16Rect::new((3, 6), (10, 15));
    let n7 = Tnode::new(None, v7, s7);
    let nid7 = n7.id();
    let n7 = Tnode::to_arc(n7);

    let v8 = TestValue { value: 8 };
    let s8 = IRect::new((-1, -2), (2, 1));
    let us8 = U16Rect::new((3, 6), (5, 7));
    let n8 = Tnode::new(None, v8, s8);
    let nid8 = n8.id();
    let n8 = Tnode::to_arc(n8);

    let v9 = TestValue { value: 9 };
    let s9 = IRect::new((5, 6), (9, 8));
    let us9 = U16Rect::new((8, 12), (10, 14));
    let n9 = Tnode::new(None, v9, s9);
    let nid9 = n9.id();
    let n9 = Tnode::to_arc(n9);

    /**
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *      /  \     \
     *     n4  n5    n6
     *           \
     *            n7
     *           / \
     *         n8   n9
     * ```
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n2.clone(), n5.clone());
    Inode::push(n3.clone(), n6.clone());
    Inode::push(n5.clone(), n7.clone());
    Inode::push(n7.clone(), n8.clone());
    Inode::push(n7.clone(), n9.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    let n3 = n3.lock();
    let n4 = n4.lock();
    let n5 = n5.lock();
    let n6 = n6.lock();
    let n7 = n7.lock();
    let n8 = n8.lock();
    let n9 = n9.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());
    info!("n7:{:?}", n7.borrow());
    info!("n8:{:?}", n8.borrow());
    info!("n9:{:?}", n9.borrow());

    let expects = vec![us1, us2, us3, us4, us5, us6, us7, us8, us9];
    let nodes = vec![n1, n2, n3, n4, n5, n6, n7, n8, n9];
    for i in 0..9 {
      let expect = expects[i];
      let node = &nodes[i];
      let actual = node.borrow().actual_shape();
      assert_eq!(expect, actual);
    }
  }

  #[test]
  fn shape2() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = TestValue { value: 1 };
    let s1 = IRect::new((0, 0), (20, 20));
    let us1 = U16Rect::new((0, 0), (20, 20));
    let n1 = Tnode::new(None, v1, s1);
    let nid1 = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = TestValue { value: 2 };
    let s2 = IRect::new((0, 0), (20, 20));
    let us2 = U16Rect::new((0, 0), (20, 20));
    let n2 = Tnode::new(None, v2, s2);
    let nid2 = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = TestValue { value: 3 };
    let s3 = IRect::new((-2, -2), (-1, 0));
    let us3 = U16Rect::new((0, 0), (0, 0));
    let n3 = Tnode::new(None, v3, s3);
    let nid3 = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = TestValue { value: 4 };
    let s4 = IRect::new((3, 5), (20, 20));
    let us4 = U16Rect::new((3, 5), (20, 20));
    let n4 = Tnode::new(None, v4, s4);
    let nid4 = n4.id();
    let n4 = Tnode::to_arc(n4);

    let v5 = TestValue { value: 5 };
    let s5 = IRect::new((-3, -5), (15, 20));
    let us5 = U16Rect::new((0, 0), (15, 20));
    let n5 = Tnode::new(None, v5, s5);
    let nid5 = n5.id();
    let n5 = Tnode::to_arc(n5);

    let v6 = TestValue { value: 5 };
    let s6 = IRect::new((8, 13), (18, 25));
    let us6 = U16Rect::new((8, 13), (15, 20));
    let n6 = Tnode::new(None, v6, s6);
    let nid6 = n6.id();
    let n6 = Tnode::to_arc(n6);

    /**
     * The tree looks like:
     * ```
     *           n1
     *         /   \
     *        n2   n3
     *         \
     *         n4
     *        /
     *       n5
     *      /
     *     n6
     * ```
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n2.clone(), n4.clone());
    Inode::push(n4.clone(), n5.clone());
    Inode::push(n5.clone(), n6.clone());

    let n1 = n1.lock();
    let n2 = n2.lock();
    let n3 = n3.lock();
    let n4 = n4.lock();
    let n5 = n5.lock();
    let n6 = n6.lock();
    info!("n1:{:?}", n1.borrow());
    info!("n2:{:?}", n2.borrow());
    info!("n3:{:?}", n3.borrow());
    info!("n4:{:?}", n4.borrow());
    info!("n5:{:?}", n5.borrow());
    info!("n6:{:?}", n6.borrow());

    let expects = vec![us1, us2, us3, us4, us5, us6];
    let nodes = vec![n1, n2, n3, n4, n5, n6];
    for i in 0..9 {
      let expect = expects[i];
      let node = &nodes[i];
      let actual = node.borrow().actual_shape();
      assert_eq!(expect, actual);
    }
  }

  #[test]
  fn push1() {
    INIT.call_once(|| {
      test_log_init();
    });

    let v1 = TestValue { value: 1 };
    let s1 = IRect::new((0, 0), (10, 10));
    let us1 = U16Rect::new((0, 0), (10, 10));
    let n1 = Tnode::new(None, v1, s1);
    let nid1 = n1.id();
    let n1 = Tnode::to_arc(n1);

    let v2 = TestValue { value: 2 };
    let s2 = IRect::new((0, 0), (10, 10));
    let us2 = U16Rect::new((0, 0), (10, 10));
    let n2 = Tnode::new(None, v2, s2);
    let nid2 = n2.id();
    let n2 = Tnode::to_arc(n2);

    let v3 = TestValue { value: 3 };
    let s3 = IRect::new((0, 0), (10, 10));
    let us3 = U16Rect::new((0, 0), (10, 10));
    let n3 = Tnode::new(None, v3, s3);
    let nid3 = n3.id();
    let n3 = Tnode::to_arc(n3);

    let v4 = TestValue { value: 4 };
    let s4 = IRect::new((0, 0), (10, 10));
    let us4 = U16Rect::new((0, 0), (10, 10));
    let n4 = Tnode::new(None, v4, s4);
    let nid4 = n4.id();
    let n4 = Tnode::to_arc(n4);

    /**
     * The tree looks like:
     * ```
     *           n1
     *         /    \
     *       n2, n3, n4
     * ```
     **/
    Inode::push(n1.clone(), n2.clone());
    Inode::push(n1.clone(), n3.clone());
    Inode::push(n1.clone(), n4.clone());

    let n1 = n1.lock();
    let n1 = n1.borrow();
    let n2 = n2.lock();
    let n2 = n2.borrow();
    let n3 = n3.lock();
    let n3 = n3.borrow();
    let n4 = n4.lock();
    let n4 = n4.borrow();

    assert_eq!(n1.children().len(), 3);
    assert!(!n1.children().is_empty());
    assert_eq!(n2.children().len(), 0);
    assert!(n2.children().is_empty());
    assert_eq!(n3.children().len(), 0);
    assert!(n3.children().is_empty());
    assert_eq!(n4.children().len(), 0);
    assert!(n4.children().is_empty());

    for (i, c) in n1.children().iter().enumerate() {
      assert_eq!(i + 2, c.lock().borrow().value().value);
    }

    let first1 = n1.children().first();
    assert!(first1.is_some());
    assert_eq!(first1.unwrap().lock().borrow().value().value, 2);

    let last1 = n1.children().last();
    assert!(last1.is_some());
    assert_eq!(last1.unwrap().lock().borrow().value().value, 4);

    assert!(n2.children().first().is_none());
    assert!(n2.children().last().is_none());
    assert!(n3.children().first().is_none());
    assert!(n3.children().last().is_none());
    assert!(n4.children().first().is_none());
    assert!(n4.children().last().is_none());
  }
}
