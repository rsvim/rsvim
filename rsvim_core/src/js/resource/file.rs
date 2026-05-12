//! File resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use std::fmt::Debug;

pub struct FileResource {
  id: ResourceId,
  data: Option<std::fs::File>,
}

impl FileResource {
  pub fn new(data: std::fs::File) -> Self {
    Self {
      id: ResourceId::next(),
      data: Some(data),
    }
  }
}

impl Resourcify for FileResource {
  fn id(&self) -> ResourceId {
    self.id
  }

  fn close(&mut self) {
    let _ = self.data.take();
  }

  fn is_closed(&self) -> bool {
    self.data.is_some()
  }
}

#[cfg(target_family = "windows")]
impl Debug for FileResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use std::os::windows::io::AsHandle;
    f.debug_struct("FileResource")
      .field(
        "as_handle",
        &self
          .data
          .as_ref()
          .map(|f| format!("{:?}", f.as_handle()))
          .unwrap_or("none".to_string()),
      )
      .finish()
  }
}

#[cfg(not(target_family = "windows"))]
impl Debug for FileResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use std::os::fd::AsFd;
    f.debug_struct("FileResource")
      .field(
        "as_fd",
        &self
          .data
          .as_ref()
          .map(|f| format!("{:?}", f.as_fd()))
          .unwrap_or("none".to_string()),
      )
      .finish()
  }
}
