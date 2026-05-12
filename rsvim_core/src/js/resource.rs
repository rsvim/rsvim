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

  pub fn add_file(&mut self, data: std::fs::File) -> ResourceId {
    let f = FileResource::new(data);
    let rid = f.id();
    self.resources.insert(f.id(), Resource::File(f));
    rid
  }

  pub fn get(&self, rid: &ResourceId) -> Option<&Resource> {
    self.resources.get(rid)
  }

  pub fn get_mut(&mut self, rid: &ResourceId) -> Option<&mut Resource> {
    self.resources.get_mut(rid)
  }
}
