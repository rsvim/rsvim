//! Window container is the root container of the VIM window widget sub-tree.

use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

#[derive(Debug, Clone, Copy)]
/// VIM Window container, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct WindowContainer {
  id: WidgetId,
}

impl WindowContainer {
  pub fn new() -> Self {
    WindowContainer { id: uuid::next() }
  }
}

impl Default for WindowContainer {
  fn default() -> Self {
    WindowContainer::new()
  }
}

impl Widget for WindowContainer {
  fn id(&self) -> WidgetId {
    self.id
  }
}
