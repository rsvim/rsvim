//! Resource.

pub mod file;
pub mod text_decoder;

use crate::prelude::*;
use crate::structural_id_impl;
use file::FileResource;
use std::fmt::Debug;
use text_decoder::TextDecoderResource;

// ResourceId start from 1.
structural_id_impl!(i32, ResourceId, 1);

/// Resourcify
pub trait Resourcify: Sized + Debug + Clone {
  fn id(&self) -> ResourceId;
}

#[derive(Debug, Clone)]
pub enum Resource {
  File(FileResource),
  TextDecoder(TextDecoderResource),
}

impl Resourcify for Resource {
  fn id(&self) -> ResourceId {
    match self {
      Resource::File(r) => r.id(),
      Resource::TextDecoder(r) => r.id(),
    }
  }
}

#[derive(Debug)]
pub struct ResourceTable {
  resources: FoldMap<ResourceId, Resource>,
}

arc_mutex_ptr!(ResourceTable);

// pub type ResourceTableKeys<'a> =
//   std::collections::btree_map::Keys<'a, ResourceId, Resource>;
// pub type ResourceTableValues<'a> =
//   std::collections::btree_map::Values<'a, ResourceId, Resource>;
// pub type ResourceTableIter<'a> =
//   std::collections::btree_map::Iter<'a, ResourceId, Resource>;

impl ResourceTable {
  pub fn new() -> Self {
    Self {
      resources: FoldMap::new(),
    }
  }

  pub fn add_file(&mut self, data: std::fs::File) -> ResourceId {
    let res = FileResource::new(data);
    let rid = res.id();
    self.resources.insert(res.id(), Resource::File(res));
    rid
  }

  pub fn add_text_decoder(&mut self, data: encoding_rs::Decoder) -> ResourceId {
    let res = TextDecoderResource::new(data);
    let rid = res.id();
    self.resources.insert(res.id(), Resource::TextDecoder(res));
    rid
  }

  pub fn get(&self, rid: &ResourceId) -> Option<&Resource> {
    self.resources.get(rid)
  }

  pub fn remove(&mut self, rid: &ResourceId) -> Option<Resource> {
    self.resources.remove(rid)
  }
}
