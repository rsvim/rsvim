//! Cursor widget.

use crate::cart::{IPos, IRect, U16Pos};
use crate::ui::frame::CursorStyle;
use crate::ui::tree::node::NodeId;
use crate::ui::widget::Widget;
use crate::uuid;
use geo::point;

pub struct Cursor {
  id: NodeId,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(pos: IPos, blinking: bool, hidden: bool, style: CursorStyle) -> Self {
    Cursor {
      id: uuid::next(),
      blinking,
      hidden,
      style,
    }
  }
}

impl Widget for Cursor {
  fn id(&self) -> NodeId {
    self.id
  }

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
