//! Root container is the root node in the widget tree.

use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

#[derive(Debug, Clone, Copy)]
/// Root container.
pub struct RootContainer {
  id: WidgetId,
}

impl RootContainer {
  pub fn new() -> Self {
    RootContainer { id: uuid::next() }
  }
}

impl Default for RootContainer {
  fn default() -> Self {
    RootContainer::new()
  }
}

impl Widget for RootContainer {
  fn id(&self) -> WidgetId {
    self.id
  }
}
