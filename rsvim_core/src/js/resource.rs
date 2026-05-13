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

#[derive(Debug, Clone)]
pub enum Resource {
  File(FileResource),
}

#[derive(Debug)]
pub struct ResourceTable {
  resources: BTreeMap<ResourceId, Resource>,
}

arc_mutex_ptr!(ResourceTable);

pub type ResourceTableKeys<'a> =
  std::collections::btree_map::Keys<'a, ResourceId, Resource>;
pub type ResourceTableValues<'a> =
  std::collections::btree_map::Values<'a, ResourceId, Resource>;
pub type ResourceTableIter<'a> =
  std::collections::btree_map::Iter<'a, ResourceId, Resource>;

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

  pub fn remove(&mut self, rid: &ResourceId) -> Option<Resource> {
    self.resources.remove(rid)
  }

  pub fn keys(&self) -> ResourceTableKeys<'_> {
    self.resources.keys()
  }

  pub fn values(&self) -> ResourceTableValues<'_> {
    self.resources.values()
  }

  pub fn iter(&self) -> ResourceTableIter<'_> {
    self.resources.iter()
  }
}
