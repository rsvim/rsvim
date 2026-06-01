//! Resource.

pub mod child_process;
pub mod file;
pub mod text_decoder;

use crate::prelude::*;
use child_process::ChildProcessResource;
use file::FileResource;
use std::fmt::Debug;
use text_decoder::TextDecoderResource;

// ResourceId start from 1.
#[derive(
  Copy, Clone, rsvim_macro::IncrementalId, serde::Serialize, serde::Deserialize,
)]
pub struct ResourceId(#[start_from(1)] i32);

/// Resourcify
pub trait Resourcify: Sized + Debug + Clone {
  fn id(&self) -> ResourceId;
}

#[derive(Debug, Clone)]
pub enum Resource {
  File(FileResource),
  TextDecoder(TextDecoderResource),
  ChildProcess(ChildProcessResource),
}

impl Resourcify for Resource {
  fn id(&self) -> ResourceId {
    match self {
      Resource::File(r) => r.id(),
      Resource::TextDecoder(r) => r.id(),
      Resource::ChildProcess(r) => r.id(),
    }
  }
}

#[derive(Debug, rsvim_macro::ArcMutexPtr)]
pub struct ResourceTable {
  resources: FoldMap<ResourceId, Resource>,
}

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

  pub fn add_child_process(&mut self, data: std::process::Child) -> ResourceId {
    let res = ChildProcessResource::new(data);
    let rid = res.id();
    self.resources.insert(res.id(), Resource::ChildProcess(res));
    rid
  }

  pub fn get(&self, rid: &ResourceId) -> Option<&Resource> {
    self.resources.get(rid)
  }

  pub fn remove(&mut self, rid: &ResourceId) -> Option<Resource> {
    self.resources.remove(rid)
  }
}
