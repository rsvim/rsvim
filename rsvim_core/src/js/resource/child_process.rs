//! Command and child-process resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use crate::prelude::*;
use std::fmt::Debug;
use std::process::Child;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
/// We name it "ProcCommand" to distinguish with the Vim ex command struct
/// "Command".
pub struct ProcCommandResource {
  id: ResourceId,
  data: Arc<Mutex<Command>>,
}

impl ProcCommandResource {
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

impl Resourcify for ProcCommandResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}

impl Debug for ProcCommandResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CommandResource")
      .field("id", &self.id)
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

impl Debug for ChildProcessResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ChildProcessResource")
      .field("id", &self.id)
      .finish()
  }
}
