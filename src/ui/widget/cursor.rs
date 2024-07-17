//! Cursor widget.

use crate::cart::{IPos, IRect, U16Pos};
use crate::define_widget_base_helpers;
use crate::ui::frame::CursorStyle;
use crate::ui::tree::NodeId;
use crate::ui::widget::{Widget, WidgetBase};
use geo::point;

pub struct Cursor {
  base: WidgetBase,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(pos: IPos, blinking: bool, hidden: bool, style: CursorStyle) -> Self {
    let rect = IRect::new(pos, pos + point!(x:1, y:1));
    let zindex = std::usize::MAX;
    let base = WidgetBase::new(rect, zindex);
    Cursor {
      base,
      blinking,
      hidden,
      style,
    }
  }
}

impl Widget for Cursor {
  define_widget_base_helpers!();

  fn draw(&mut self) {
    let abs_rect_min = self.absolute_rect().min();
    let pos: U16Pos = point! (x: abs_rect_min.x as u16, y: abs_rect_min.y as u16);

    let frame = self.terminal().frame_mut();
    frame.set_cursor(crate::ui::frame::Cursor::new(
      pos,
      self.blinking,
      self.hidden,
      self.style,
    ));
  }
}
