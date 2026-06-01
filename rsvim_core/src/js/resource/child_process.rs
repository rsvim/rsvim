//! Command child-process resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use crate::prelude::*;
use std::process::Child;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

#[derive_where::derive_where(Debug)]
#[derive(Clone)]
pub struct ChildProcessResource {
  id: ResourceId,
  #[derive_where(skip)]
  data: Arc<Mutex<Command>>,
}

impl ChildProcessResource {
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

impl Resourcify for ChildProcessResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}

#[derive_where::derive_where(Debug)]
#[derive(Clone)]
pub struct ChildProcessResource {
  id: ResourceId,
  #[derive_where(skip)]
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
