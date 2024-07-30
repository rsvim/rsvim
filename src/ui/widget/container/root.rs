//! Root container is the root node in the widget tree.

use crate::ui::widget::Widget;

/// Root container.
#[derive(Debug, Clone, Copy, Default)]
pub struct RootContainer {}

impl RootContainer {
  pub fn new() -> Self {
    RootContainer {}
  }
}

impl Widget for RootContainer {}
