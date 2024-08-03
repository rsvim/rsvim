//! The node structure of the internal tree.

use std::fmt::Debug;

use geo::point;

use crate::cart::{IRect, U16Rect};
use crate::geo_rect_as;

pub type InodeId = usize;

pub trait InodeValue: Sized + Clone + Debug {
  fn id(&self) -> InodeId;
}

#[derive(Debug, Clone)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct Inode<T>
where
  T: InodeValue,
{
  value: T,
  depth: usize,
  shape: IRect,
  actual_shape: U16Rect,
  zindex: usize,
  enabled: bool,
  visible: bool,
}

impl<T> Default for Inode<T>
where
  T: InodeValue,
{
  fn default() -> Self {
    let shape = IRect::new((0, 0), (0, 0));
    let actual_shape = U16Rect::new((0, 0), (0, 0));
    Inode {
      value: Default::default(),
      depth: 0,
      shape,
      actual_shape,
      zindex: 0,
      enabled: true,
      visible: true,
    }
  }
}

impl<T> Inode<T>
where
  T: InodeValue,
{
  pub fn new(value: T, shape: IRect) -> Self {
    let actual_shape = geo_rect_as!(shape, u16);
    Inode {
      value,
      depth: 0,
      shape,
      actual_shape,
      zindex: 0,
      enabled: true,
      visible: true,
    }
  }

  pub fn id(&self) -> InodeId {
    self.value.id()
  }

  pub fn value(&self) -> &T {
    &self.value
  }

  pub fn value_mut(&mut self) -> &mut T {
    &mut self.value
  }

  pub fn depth(&self) -> &usize {
    &self.depth
  }

  pub fn depth_mut(&mut self) -> &mut usize {
    &mut self.depth
  }

  pub fn zindex(&self) -> &usize {
    &self.zindex
  }

  pub fn zindex_mut(&mut self) -> &mut usize {
    &mut self.zindex
  }

  pub fn shape(&self) -> &IRect {
    &self.shape
  }

  pub fn shape_mut(&mut self) -> &mut IRect {
    &mut self.shape
  }

  pub fn actual_shape(&self) -> &U16Rect {
    &self.actual_shape
  }

  pub fn actual_shape_mut(&mut self) -> &mut U16Rect {
    &mut self.actual_shape
  }

  pub fn enabled(&self) -> &bool {
    &self.enabled
  }

  pub fn enabled_mut(&mut self) -> &mut bool {
    &mut self.enabled
  }

  pub fn visible(&self) -> &bool {
    &self.visible
  }

  pub fn visible_mut(&mut self) -> &mut bool {
    &mut self.visible
  }
}

#[cfg(test)]
mod tests {
  use std::cell::RefCell;
  use std::sync::Once;

  use geo::CoordNum;
  use geo::Rect;

  use crate::cart::IRect;
  use crate::test::log::init as test_log_init;

  use super::*;

  // Test node
  #[derive(Default, Copy, Clone, Debug)]
  struct Tvalue {
    value: usize,
  }

  impl InodeValue for Tvalue {
    fn id(&self) -> InodeId {
      self.value
    }
  }

  type Tnode = Inode<Tvalue>;

  static INIT: Once = Once::new();

  fn shape_eq<T: CoordNum>(s1: Rect<T>, s2: Rect<T>) -> bool {
    s1.min() == s2.min() && s1.max() == s2.max()
  }

  #[test]
  fn new() {
    INIT.call_once(|| {
      test_log_init();
    });

    let n1 = Tnode::default();
    let n2 = Tnode::new(Tvalue { value: 2 }, IRect::new((1, 2), (3, 4)));
    let n1 = RefCell::new(n1);
    let n2 = RefCell::new(n2);

    assert_eq!(n1.borrow().id(), 0);
    assert_eq!(n2.borrow().id(), 2);
    assert_eq!(n1.borrow().value().value, 0);
    assert_eq!(n2.borrow().value().value, 2);

    n1.borrow_mut().value_mut().value = 3;
    n2.borrow_mut().value_mut().value = 4;
    assert_eq!(n1.borrow().id(), 3);
    assert_eq!(n2.borrow().id(), 4);
    assert_eq!(n1.borrow().value().value, 3);
    assert_eq!(n2.borrow().value().value, 4);

    assert_eq!(*n1.borrow().depth(), 0);
    assert_eq!(*n1.borrow().zindex(), 0);
    assert!(n1.borrow().enabled());
    assert!(n1.borrow().visible());

    assert!(shape_eq(*n1.borrow().shape(), IRect::new((0, 0), (0, 0))));
    assert!(shape_eq(*n2.borrow().shape(), IRect::new((1, 2), (3, 4))));
  }
}
