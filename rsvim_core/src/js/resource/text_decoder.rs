//! Text decoder resource.

use crate::js::resource::ResourceId;
use crate::js::resource::Resourcify;
use std::sync::Arc;
use std::sync::Mutex;

#[derive_where::derive_where(Debug)]
#[derive(Clone)]
pub struct TextDecoderResource {
  id: ResourceId,
  #[derive_where(skip)]
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
