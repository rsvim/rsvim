//! Resource.

pub mod file;

use crate::prelude::*;
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

pub struct ResourceTable {
  resources: BTreeMap<ResourceId, Resource>,
}

impl ResourceTable {
  pub fn new() -> Self {
    Self {
      resources: BTreeMap::new(),
    }
  }
}
