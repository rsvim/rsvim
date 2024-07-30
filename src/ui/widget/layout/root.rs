//! Root layout is the root node in the widget tree.

use crate::ui::widget::Widget;

/// Root layout.
#[derive(Debug, Clone, Copy, Default)]
pub struct RootLayout {}

impl RootLayout {
  pub fn new() -> Self {
    RootLayout {}
  }
}

impl Widget for RootLayout {}
