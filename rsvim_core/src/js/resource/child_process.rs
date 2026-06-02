//! Command child-process resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use std::process::Child;
use std::process::ChildStderr;
use std::process::ChildStdin;
use std::process::ChildStdout;
use std::sync::Arc;
use std::sync::Mutex;

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

#[derive_where::derive_where(Debug)]
#[derive(Clone)]
pub struct ChildProcessStdinResource {
  id: ResourceId,
  #[derive_where(skip)]
  data: Arc<Mutex<ChildStdin>>,
}

impl ChildProcessStdinResource {
  pub fn new(data: ChildStdin) -> Self {
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn data(&self) -> Arc<Mutex<ChildStdin>> {
    self.data.clone()
  }
}

impl Resourcify for ChildProcessStdinResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}

#[derive_where::derive_where(Debug)]
#[derive(Clone)]
pub struct ChildProcessStdoutResource {
  id: ResourceId,
  #[derive_where(skip)]
  data: Arc<Mutex<ChildStdout>>,
}

impl ChildProcessStdoutResource {
  pub fn new(data: ChildStdout) -> Self {
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn data(&self) -> Arc<Mutex<ChildStdout>> {
    self.data.clone()
  }
}

impl Resourcify for ChildProcessStdoutResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}

#[derive_where::derive_where(Debug)]
#[derive(Clone)]
pub struct ChildProcessStderrResource {
  id: ResourceId,
  #[derive_where(skip)]
  data: Arc<Mutex<ChildStderr>>,
}

impl ChildProcessStderrResource {
  pub fn new(data: ChildStderr) -> Self {
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn data(&self) -> Arc<Mutex<ChildStderr>> {
    self.data.clone()
  }
}

impl Resourcify for ChildProcessStderrResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}
