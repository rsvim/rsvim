//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use crate::ui::tree::node::NodeId;
use crate::ui::widget::Widget;
use crate::uuid;

/// Root widget.
pub struct RootWidget {
  id: NodeId,
}

impl RootWidget {
  pub fn new() -> Self {
    RootWidget { id: uuid::next() }
  }
}

impl Widget for RootWidget {
  fn id(&self) -> NodeId {
    self.id
  }
}
