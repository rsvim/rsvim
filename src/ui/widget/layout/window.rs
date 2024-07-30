//! Window layout is the root node of the VIM window widget sub-tree.

use crate::ui::widget::Widget;

/// VIM Window layout, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
#[derive(Debug, Clone, Copy)]
pub struct WindowLayout {}

impl Widget for WindowLayout {}
