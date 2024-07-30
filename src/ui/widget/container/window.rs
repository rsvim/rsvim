//! Window container is the root container of the VIM window widget sub-tree.

use crate::ui::widget::Widget;

/// VIM Window container, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowContainer {}

impl Widget for WindowContainer {}
