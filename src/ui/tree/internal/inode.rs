//! Internal tree structure implementation: the `Inode` structure.

use std::sync::{Arc, RwLock, Weak};

use crate::cart::{IRect, U16Rect};
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

  pub fn parent(&self) -> Option<InodeWk<T>> {
    self.parent
  }

  pub fn children(&self) -> Option<&Vec<InodePtr<T>>> {
    self.children
  }

  pub fn attribute(&self) -> InodeAttr {
    self.attr
  }

  pub fn value(&self) -> &T {
    &self.value
  }

  pub fn get_child(&self, index: usize) -> Option<&InodePtr<T>> {
    match self.children {
      Some(c) => c.get(index),
      None => None,
    }
  }

  pub fn add_child(&mut self, child: InodePtr<T>) {
    if self.children.is_none() {
      self.children = Some(vec![]);
    }

    match self.children {
      Some(&mut c) => c.push(child),
      None => unreachable!(),
    }
  }

  pub fn remove_child(&mut self, index: usize) -> Option<&InodePtr<T>> {
    match self.children {
      Some(c) => c.remove(index),
      None => None,
    }
  }
}
