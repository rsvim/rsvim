//! Root layout is the root container for all other UI widgets in the widget tree, it exists along
//! with the widget tree.

use crate::ui::tree::node::NodeId;
use crate::ui::widget::{Layout, Widget};
use crate::uuid;

/// Root layout.
#[derive(Debug, Clone, Copy)]
pub struct RootLayout {
  id: NodeId,
}

impl RootLayout {
  pub fn new() -> Self {
    RootLayout { id: uuid::next() }
  }

  pub fn id(&self) -> NodeId {
    self.id
  }
}

impl Default for RootLayout {
  fn default() -> Self {
    RootLayout { id: uuid::next() }
  }
}

impl Widget for RootLayout {
  fn id(&self) -> NodeId {
    RootLayout::id(&self)
  }
}

impl Layout for RootLayout {
  fn id(&self) -> NodeId {
    RootLayout::id(&self)
  }
}
