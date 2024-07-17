//! The VIM Window.

use crate::cart::IRect;
use crate::define_widget_base_helpers;
use crate::ui::tree::NodeId;
use crate::ui::widget::{Widget, WidgetBase};

/// The VIM window.
pub struct Window {
  base: WidgetBase,
}

impl Window {
  pub fn new(rect: IRect, zindex: usize) -> Self {
    let base = WidgetBase::new(rect, zindex);
    Window { base }
  }
}

impl Widget for Window {
  define_widget_base_helpers!();

  fn draw(&mut self) {
    todo!();
  }
}
