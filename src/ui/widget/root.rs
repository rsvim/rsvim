//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use crate::cart::{IRect, U16Size};
use crate::define_widget_base_helpers;
use crate::ui::tree::NodeId;
use crate::ui::widget::{Widget, WidgetBase};

/// Root widget.
pub struct RootWidget {
  base: WidgetBase,
}

impl RootWidget {
  pub fn new(terminal_size: U16Size) -> Self {
    let rect = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    let zindex = 0;
    let base = WidgetBase::new(rect, zindex);
    RootWidget { base }
  }
}

impl Widget for RootWidget {
  define_widget_base_helpers!();

  fn draw(&mut self) {
    todo!();
  }
}
