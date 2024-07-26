//! Root layout is the root node in a tree, it only has a shape without actual content, it's a
//! logical container that manages all its nested children widget nodes,

use crate::ui::tree::node::NodeId;
use crate::ui::widget::Widget;
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
}

impl Default for RootLayout {
  fn default() -> Self {
    RootLayout { id: uuid::next() }
  }
}

impl Widget for RootLayout {
  fn id(&self) -> NodeId {
    self.id
  }
}
