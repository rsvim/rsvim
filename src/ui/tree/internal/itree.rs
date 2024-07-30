//! Internal tree structure implementation: the `Itree` structure.

use crate::cart::{IRect, ISize, Size, U16Rect, U16Size};
use crate::geo_size_as;
use crate::ui::tree::internal::inode::{Inode, InodeAttr, InodePtr};

#[derive(Debug, Clone)]
pub struct Itree<T> {
  root: Option<InodePtr<T>>,
}

impl<T> Itree<T> {
  pub fn new(shape: IRect) -> Self {
    let isz = ISize::from(shape);
    let shape = IRect::new((0, 0), (isz.width(), isz.height()));
    let usz: U16Size = geo_size_as!(isz, u16);
    let actual_shape = U16Rect::new((0, 0), (usz.width(), usz.height()));
    let attr = InodeAttr::new(1, shape, actual_shape);
    let node = Inode::new(None, value, attr);
    Itree {
      root: Some(Inode::ptr(node)),
    }
  }
}
