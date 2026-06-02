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
pub struct ChildProcessResourceData {
  #[derive_where(skip)]
  pub child: Child,
  #[derive_where(skip)]
  pub stdout: Option<ChildStdout>,
  #[derive_where(skip)]
  pub stderr: Option<ChildStderr>,
}

#[derive_where::derive_where(Debug)]
#[derive(Clone)]
pub struct ChildProcessResource {
  id: ResourceId,
  #[derive_where(skip)]
  data: Arc<Mutex<ChildProcessResourceData>>,
}

impl ChildProcessResource {
  pub fn new(mut data: Child) -> Self {
    let stdout = data.stdout.take();
    let stderr = data.stderr.take();
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(ChildProcessResourceData {
        child: data,
        stdout,
        stderr,
      })),
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
