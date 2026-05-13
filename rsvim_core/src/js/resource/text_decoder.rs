//! Text decoder resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct TextDecoderResource {
  id: ResourceId,
  data: Arc<Mutex<encoding_rs::Decoder>>,
}

impl TextDecoderResource {
  pub fn new(data: encoding_rs::Decoder) -> Self {
    Self {
      id: ResourceId::next(),
      data: Arc::new(Mutex::new(data)),
    }
  }

  pub fn data(&self) -> Arc<Mutex<encoding_rs::Decoder>> {
    self.data.clone()
  }
}

impl Resourcify for TextDecoderResource {
  fn id(&self) -> ResourceId {
    self.id
  }
}

impl Debug for TextDecoderResource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TextDecoderResource")
      .field("id", &self.id)
      .finish()
  }
}
