//! Cursor widget.

use crate::cart::{IPos, IRect, U16Pos, USize};
use crate::ui::frame::CursorStyle;
use crate::ui::tree::node::NodeId;
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
  fn id(&self) -> NodeId {
    self.base.id()
  }

  fn rect(&self) -> IRect {
    self.base.rect()
  }

  fn set_rect(&mut self, rect: IRect) {
    // The rect size is always (1, 1), i.e. both height and width are always 1.
    assert_eq!(rect.min().x, rect.max().x - 1);
    assert_eq!(rect.min().y, rect.max().y - 1);
    self.base.rect = rect;
  }

  fn zindex(&self) -> usize {
    self.base.zindex()
  }

  fn set_zindex(&mut self, zindex: usize) {
    self.base.set_zindex(zindex);
  }

  fn visible(&self) -> bool {
    self.base.visible()
  }

  fn set_visible(&mut self, value: bool) {
    self.base.set_visible(value);
  }

  fn enabled(&self) -> bool {
    self.base.enabled()
  }

  fn set_enabled(&mut self, value: bool) {
    self.base.set_enabled(value);
  }

  fn set_pos(&mut self, pos: IPos) {
    self.set_rect(IRect::new(pos, pos + point!(x: 1, y: 1)));
  }

  fn set_size(&mut self, _sz: USize) {
    unimplemented!();
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
