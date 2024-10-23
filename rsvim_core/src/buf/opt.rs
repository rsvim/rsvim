//! Both global and local options for buffers.

use crate::defaults;

#[derive(Debug, Clone)]
pub struct BufferGlobalOptions {
  tab_stop: u16,
}

impl Default for BufferGlobalOptions {
  fn default() -> Self {
    Self::builder().build()
  }
}

impl BufferGlobalOptions {
  pub fn builder() -> BufferGlobalOptionsBuilder {
    BufferGlobalOptionsBuilder::default()
  }

  pub fn tab_stop(&self) -> u16 {
    self.tab_stop
  }

  pub fn set_tab_stop(&mut self, value: u16) {
    self.tab_stop = value;
  }
}

#[derive(Debug, Clone)]
/// Global window options builder.
pub struct BufferGlobalOptionsBuilder {
  tab_stop: u16,
}

impl BufferGlobalOptionsBuilder {
  pub fn tab_stop(&mut self, value: u16) -> &mut Self {
    self.tab_stop = value;
    self
  }

  pub fn build(&self) -> BufferGlobalOptions {
    BufferGlobalOptions {
      tab_stop: self.tab_stop,
    }
  }
}

impl Default for BufferGlobalOptionsBuilder {
  fn default() -> Self {
    BufferGlobalOptionsBuilder {
      tab_stop: defaults::buf::TAB_STOP,
    }
  }
}
