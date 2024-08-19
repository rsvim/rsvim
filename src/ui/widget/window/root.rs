//! Window root container.

use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

#[derive(Debug, Clone, Copy)]
/// Window root container.
pub struct WindowRootContainer {
  id: WidgetId,
}

impl WindowRootContainer {
  pub fn new() -> Self {
    WindowRootContainer { id: uuid::next() }
  }
}

impl Default for WindowRootContainer {
  fn default() -> Self {
    WindowRootContainer::new()
  }
}

impl Widget for WindowRootContainer {
  fn id(&self) -> WidgetId {
    self.id
  }
}
