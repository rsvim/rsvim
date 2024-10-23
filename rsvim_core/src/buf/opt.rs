//! Both global and local options for buffers.

use crate::defaults;

#[derive(Debug, Clone)]
pub struct BufferLocalOptions {
  tab_stop: u16,
}

impl Default for BufferLocalOptions {
  fn default() -> Self {
    Self::builder().build()
  }
}

impl BufferLocalOptions {
  pub fn builder() -> BufferLocalOptionsBuilder {
    BufferLocalOptionsBuilder::default()
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
pub struct BufferLocalOptionsBuilder {
  tab_stop: u16,
}

impl BufferLocalOptionsBuilder {
  pub fn tab_stop(&mut self, value: u16) -> &mut Self {
    self.tab_stop = value;
    self
  }

  pub fn build(&self) -> BufferLocalOptions {
    BufferLocalOptions {
      tab_stop: self.tab_stop,
    }
  }
}

impl Default for BufferLocalOptionsBuilder {
  fn default() -> Self {
    BufferLocalOptionsBuilder {
      tab_stop: defaults::buf::TAB_STOP,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default1() {
    let opt1 = BufferLocalOptions::default();
    let opt2 = BufferLocalOptionsBuilder::default().build();
    assert_eq!(opt1.tab_stop(), opt2.tab_stop());
  }
}
