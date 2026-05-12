//! File resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use std::fmt::Debug;

pub struct FileResource {
  id: ResourceId,
  data: std::fs::File,
}

impl FileResource {
  pub fn new(data: std::fs::File) -> Self {
    Self {
      id: ResourceId::next(),
      data,
    }
  }

  pub fn data(&self) -> &std::fs::File {
    &self.data
  }

  pub fn data_mut(&mut self) -> &mut std::fs::File {
    &mut self.data
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
      .field("as_handle", &self.data.as_handle())
      .finish()
  }
}

#[cfg(not(target_family = "windows"))]
impl Debug for FileResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use std::os::fd::AsFd;
    f.debug_struct("FileResource")
      .field("as_fd", &self.data.as_fd())
      .finish()
  }
}
