//! The node structure of the internal tree.

use geo;
use std::fmt::Debug;

use crate::cart::{IRect, U16Rect};
use crate::{geo_rect_as, uuid};

pub type InodeId = usize;

pub trait Inodeable: Sized + Clone + Debug {
  fn id(&self) -> InodeId;

  fn depth(&self) -> &usize;

  fn depth_mut(&mut self) -> &mut usize;

  fn zindex(&self) -> &usize;

  fn zindex_mut(&mut self) -> &mut usize;

  fn shape(&self) -> &IRect;

  fn shape_mut(&mut self) -> &mut IRect;

  fn actual_shape(&self) -> &U16Rect;

  fn actual_shape_mut(&mut self) -> &mut U16Rect;

  fn enabled(&self) -> &bool;

  fn enabled_mut(&mut self) -> &mut bool;

  fn visible(&self) -> &bool;

  fn visible_mut(&mut self) -> &mut bool;
}

/// Generate getter/setter for Inode.
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

      fn depth_mut(&mut self) -> &mut usize {
        self.$base_name.depth_mut()
      }

      fn zindex(&self) -> &usize {
        self.$base_name.zindex()
      }

      fn zindex_mut(&mut self) -> &mut usize {
        self.$base_name.zindex_mut()
      }

      fn shape(&self) -> &IRect {
        self.$base_name.shape()
      }

      fn shape_mut(&mut self) -> &mut IRect {
        self.$base_name.shape_mut()
      }

      fn actual_shape(&self) -> &U16Rect {
        self.$base_name.actual_shape()
      }

      fn actual_shape_mut(&mut self) -> &mut U16Rect {
        self.$base_name.actual_shape_mut()
      }

      fn enabled(&self) -> &bool {
        self.$base_name.enabled()
      }

      fn enabled_mut(&mut self) -> &mut bool {
        self.$base_name.enabled_mut()
      }

      fn visible(&self) -> &bool {
        self.$base_name.visible()
      }

      fn visible_mut(&mut self) -> &mut bool {
        self.$base_name.visible_mut()
      }
    }
  };
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
      id: uuid::next(),
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

  use crate::cart::IRect;
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

    assert_eq!(n1.borrow().id() + 1, n2.borrow().id());
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
}
