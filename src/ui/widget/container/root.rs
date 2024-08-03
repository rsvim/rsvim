//! Root container is the root node in the widget tree.

use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

/// Root container.
#[derive(Debug, Clone, Copy)]
pub struct RootContainer {
  id: WidgetId,
}

impl RootContainer {
  pub fn new() -> Self {
    RootContainer { id: uuid::next() }
  }
}

impl Widget for RootContainer {
  fn id(&self) -> WidgetId {
    self.id
  }
}
