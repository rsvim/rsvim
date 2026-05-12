//! Resource management.

use crate::structural_id_impl;
use std::fmt::Debug;

// ResourceId start from 1.
structural_id_impl!(i32, ResourceId, 1);

/// All resources are closable.
pub trait Closable: Sized + Debug {
  fn close();

  fn is_closed() -> bool;
}

pub enum Resource {}

pub struct FileDescriptor {}

#[derive(Debug)]
pub struct ChildProcess {}

pub struct ResourceTable {}
