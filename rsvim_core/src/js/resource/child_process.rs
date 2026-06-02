//! Command child-process resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use std::process::Child;
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
  pub fn new(mut data: Child) -> Self {
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
