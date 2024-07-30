//! Root layout is the root node in the widget tree.

use crate::ui::widget::Widget;

/// Root layout.
#[derive(Debug, Clone, Copy)]
pub struct RootLayout {}

impl Widget for RootLayout {}
