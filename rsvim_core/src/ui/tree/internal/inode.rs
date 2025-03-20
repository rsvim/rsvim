//! The node structure of the internal tree.

use crate::geo_rect_as;
use crate::prelude::*;

use geo;
use std::fmt::Debug;
use std::sync::atomic::{AtomicI32, Ordering};

pub type InodeId = i32;

pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> InodeId;

  fn depth(&self) -> &usize;

  fn set_depth(&mut self, depth: usize);

  fn zindex(&self) -> &usize;

  fn set_zindex(&mut self, zindex: usize);

  fn shape(&self) -> &IRect;

  fn set_shape(&mut self, shape: &IRect);

  fn actual_shape(&self) -> &U16Rect;

  fn set_actual_shape(&mut self, actual_shape: &U16Rect);

  fn enabled(&self) -> &bool;

  fn set_enabled(&mut self, enabled: bool);

  fn visible(&self) -> &bool;

  fn set_visible(&mut self, visible: bool);
}

/// Generate getter/setter for `Inode`.
#[macro_export]
macro_rules! inode_generate_impl {
  ($struct_name:ty,$base_name:ident) => {
    impl Inodeable for $struct_name {
      fn id(&self) -> InodeId {
        self.$base_name.id()
      }

      fn depth(&self) -> &usize {
        self.$base_name.depth()
      }

      fn set_depth(&mut self, depth: usize) {
        self.$base_name.set_depth(depth);
      }

      fn zindex(&self) -> &usize {
        self.$base_name.zindex()
      }

      fn set_zindex(&mut self, zindex: usize) {
        self.$base_name.set_zindex(zindex);
      }

      fn shape(&self) -> &IRect {
        self.$base_name.shape()
      }

      fn set_shape(&mut self, shape: &IRect) {
        self.$base_name.set_shape(shape);
      }

      fn actual_shape(&self) -> &U16Rect {
        self.$base_name.actual_shape()
      }

      fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
        self.$base_name.set_actual_shape(actual_shape)
      }

      fn enabled(&self) -> &bool {
        self.$base_name.enabled()
      }

      fn set_enabled(&mut self, enabled: bool) {
        self.$base_name.set_enabled(enabled);
      }

      fn visible(&self) -> &bool {
        self.$base_name.visible()
      }

      fn set_visible(&mut self, visible: bool) {
        self.$base_name.set_visible(visible);
      }
    }
  };
}

/// Next unique UI widget ID.
///
/// NOTE: Start from 100001, so be different from buffer ID.
pub fn next_node_id() -> InodeId {
  static VALUE: AtomicI32 = AtomicI32::new(100001);
  VALUE.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, Copy)]
/// The internal tree node, it's both a container for the widgets and common attributes.
pub struct InodeBase {
  id: InodeId,
  depth: usize,
  shape: IRect,
  actual_shape: U16Rect,
  zindex: usize,
  enabled: bool,
  visible: bool,
}

impl InodeBase {
  pub fn new(shape: IRect) -> Self {
    let actual_shape = geo_rect_as!(shape, u16);
    InodeBase {
      id: next_node_id(),
      depth: 0,
      shape,
      actual_shape,
      zindex: 0,
      enabled: true,
      visible: true,
    }
  }

  pub fn id(&self) -> InodeId {
    self.id
  }

  pub fn depth(&self) -> &usize {
    &self.depth
  }

  pub fn set_depth(&mut self, depth: usize) {
    self.depth = depth;
  }

  pub fn zindex(&self) -> &usize {
    &self.zindex
  }

  pub fn set_zindex(&mut self, zindex: usize) {
    self.zindex = zindex;
  }

  pub fn shape(&self) -> &IRect {
    &self.shape
  }

  pub fn set_shape(&mut self, shape: &IRect) {
    self.shape = *shape;
  }

  pub fn actual_shape(&self) -> &U16Rect {
    &self.actual_shape
  }

  pub fn set_actual_shape(&mut self, actual_shape: &U16Rect) {
    self.actual_shape = *actual_shape;
  }

  pub fn enabled(&self) -> &bool {
    &self.enabled
  }

  pub fn set_enabled(&mut self, enabled: bool) {
    self.enabled = enabled;
  }

  pub fn visible(&self) -> &bool {
    &self.visible
  }

  pub fn set_visible(&mut self, visible: bool) {
    self.visible = visible;
  }
}

#[cfg(test)]
mod tests {
  use std::cell::RefCell;

  use crate::prelude::*;
  // use crate::test::log::init as test_log_init;

  use super::*;

  // Test node
  #[derive(Copy, Clone, Debug)]
  struct TestNode {
    pub value: usize,
    pub base: InodeBase,
  }

  impl TestNode {
    pub fn new(value: usize, shape: IRect) -> Self {
      TestNode {
        value,
        base: InodeBase::new(shape),
      }
    }
  }

  inode_generate_impl!(TestNode, base);

  #[test]
  fn new() {
    // test_log_init();

    let n1 = TestNode::new(1, IRect::new((0, 0), (0, 0)));
    let n2 = TestNode::new(2, IRect::new((1, 2), (3, 4)));
    let n1 = RefCell::new(n1);
    let n2 = RefCell::new(n2);

    assert!(n1.borrow().id() < n2.borrow().id());
    assert_eq!(n1.borrow().value, 1);
    assert_eq!(n2.borrow().value, 2);

    n1.borrow_mut().value = 3;
    n2.borrow_mut().value = 4;
    assert_eq!(n1.borrow().value, 3);
    assert_eq!(n2.borrow().value, 4);

    assert_eq!(*n1.borrow().depth(), 0);
    assert_eq!(*n1.borrow().zindex(), 0);
    assert!(n1.borrow().enabled());
    assert!(n1.borrow().visible());

    assert_eq!(*n1.borrow().shape(), IRect::new((0, 0), (0, 0)));
    assert_eq!(*n2.borrow().shape(), IRect::new((1, 2), (3, 4)));
  }

  #[test]
  fn next_node_id1() {
    assert!(next_node_id() > 0);
  }
}
