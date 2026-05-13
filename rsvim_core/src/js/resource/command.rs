//! Command resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use crate::prelude::*;
use itertools::Itertools;
use std::fmt::Debug;
use std::process::Child;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct CommandResource {
  id: ResourceId,
  data: Arc<Mutex<Command>>,
}

impl CommandResource {
  pub fn new(data: Command) -> Self {
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn data(&self) -> Arc<Mutex<Command>> {
    self.data.clone()
  }
}

impl Resourcify for CommandResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}

impl Debug for CommandResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let data = lock!(self.data);
    f.debug_struct("CommandResource")
      .field("id", &self.id)
      .field("program", &data.get_program().to_string_lossy())
      .field(
        "args",
        &data
          .get_args()
          .into_iter()
          .map(|a| a.to_string_lossy().clone())
          .collect_vec(),
      )
      .field("cwd", &data.get_current_dir())
      .field(
        "envs",
        &data
          .get_envs()
          .into_iter()
          .map(|e| {
            format!(
              "{}={}",
              e.0.to_string_lossy().to_string(),
              e.1
                .map(|v| v.to_string_lossy().to_string())
                .unwrap_or("".to_string())
            )
          })
          .collect_vec(),
      )
      .finish()
  }
}

#[derive(Clone)]
pub struct ChildProcessResource {
  id: ResourceId,
  data: Arc<Mutex<Child>>,
}

impl ChildProcessResource {
  pub fn new(data: Child) -> Self {
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn data(&self) -> Arc<Mutex<Child>> {
    self.data.clone()
  }
}

impl Resourcify for ChildProcessResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}

#[cfg(target_family = "windows")]
impl Debug for ChildProcessResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use std::os::windows::io::AsHandle;
    f.debug_struct("ChildProcessResource")
      .field("id", &self.id)
      .field("as_handle", &lock!(self.data).as_handle())
      .finish()
  }
}

#[cfg(not(target_family = "windows"))]
impl Debug for ChildProcessResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use std::os::fd::AsFd;
    f.debug_struct("ChildProcessResource")
      .field("id", &self.id)
      .field("as_fd", &lock!(self.data).as_fd())
      .finish()
  }
}
