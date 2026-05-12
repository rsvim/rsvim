//! Resource.

pub mod file;

use crate::structural_id_impl;
use file::FileResource;
use std::fmt::Debug;

// ResourceId start from 1.
structural_id_impl!(i32, ResourceId, 1);

/// Resourcify
pub trait Resourcify: Sized + Debug {
  /// Resource ID.
  fn id(&self) -> ResourceId;

  /// All resources are closable.
  fn close(&mut self);

  fn is_closed(&self) -> bool;
}

pub enum Resource {
  File(FileResource),
}

pub struct ResourceTable {}
