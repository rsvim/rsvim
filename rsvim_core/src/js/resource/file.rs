//! File resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use crate::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

#[derive_where::derive_where(Debug)]
#[derive(Clone)]
pub struct FileResource {
  id: ResourceId,
  #[derive_where(skip)]
  data: Arc<Mutex<std::fs::File>>,
}

impl FileResource {
  pub fn new(data: std::fs::File) -> Self {
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn data(&self) -> Arc<Mutex<std::fs::File>> {
    self.data.clone()
  }
}

impl Resourcify for FileResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}
