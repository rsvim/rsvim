//! File resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use crate::prelude::*;
use std::fmt::Debug;
use std::fs::File;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct FileResource {
  id: ResourceId,
  data: Arc<Mutex<File>>,
}

impl FileResource {
  pub fn new(data: File) -> Self {
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn data(&self) -> Arc<Mutex<File>> {
    self.data.clone()
  }
}

impl Resourcify for FileResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}

#[cfg(target_family = "windows")]
impl Debug for FileResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use std::os::windows::io::AsHandle;
    f.debug_struct("FileResource")
      .field("id", &self.id)
      .field("as_handle", &lock!(self.data).as_handle())
      .finish()
  }
}

#[cfg(not(target_family = "windows"))]
impl Debug for FileResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use std::os::fd::AsFd;
    f.debug_struct("FileResource")
      .field("id", &self.id)
      .field("as_fd", &lock!(self.data).as_fd())
      .finish()
  }
}
