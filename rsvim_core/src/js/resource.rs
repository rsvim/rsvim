//! Resource.

pub mod file;

use crate::structural_id_impl;
use file::FileResource;
use std::fmt::Debug;

// ResourceId start from 1.
structural_id_impl!(i32, ResourceId, 1);

/// Resourcify
pub trait Resourcify: Sized + Debug {
  fn id(&self) -> ResourceId;
}

pub enum Resource {
  File(FileResource),
}

pub struct ResourceTable {}
