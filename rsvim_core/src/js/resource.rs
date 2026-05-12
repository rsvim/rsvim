//! Resource management.

use crate::structural_id_impl;
use std::fmt::Debug;

// ResourceId start from 1.
structural_id_impl!(i32, ResourceId, 1);

/// All resources are closable.
pub trait Closable: Sized + Debug {
  fn close(&mut self);

  fn is_closed(&self) -> bool;
}

pub enum Resource {}

pub struct FileResource {
  data: Option<std::fs::File>,
}

impl FileResource {
  pub fn new(data: std::fs::File) -> Self {
    Self { data: Some(data) }
  }
}

impl Closable for FileResource {
  fn close(&mut self) {
    self.data.take();
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

pub struct ChildProcessResource {
  data: Option<std::process::Child>,
}

pub struct ResourceTable {}
