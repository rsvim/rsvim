//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use crate::ui::tree::node::NodeId;
use crate::ui::widget::{Layout, Widget};
use crate::uuid;

/// Root widget.
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
