//! Cursor widget.

use crate::geo::{IRect, U16Pos, URect};
use crate::ui::frame::CursorStyle;
use crate::ui::term::Terminal;
use crate::ui::widget::{ChildWidgetsArc, Widget, WidgetArc};
use crate::uuid;
use geo::coord;

pub struct Cursor {
  parent: WidgetArc,
  id: usize,
  rect: IRect,
  abs_rect: URect,
  visible: bool,
  enabled: bool,

  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(
    parent: WidgetArc,
    rect: IRect,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> Self {
    Cursor {
      parent,
      id: uuid::next(),
      rect,
      abs_rect: URect::new(coord! {x:0,y:0}, coord! {x:0,y:0}),
      visible: true,
      enabled: true,

      blinking,
      hidden,
      style,
    }
  }
}

impl Widget for Cursor {
  fn id(&self) -> usize {
    self.id
  }

  fn rect(&self) -> IRect {
    self.rect
  }

  fn set_rect(&mut self, rect: IRect) {
    self.rect = rect;
  }

  fn abs_rect(&self) -> URect {
    self.abs_rect
  }

  fn set_abs_rect(&mut self, rect: URect) {
    self.abs_rect = rect;
  }

  fn zindex(&self) -> usize {
    0
  }

  fn set_zindex(&mut self, _zindex: usize) {}

  fn visible(&self) -> bool {
    self.visible
  }

  fn set_visible(&mut self, value: bool) {
    self.visible = value;
  }

  fn enabled(&self) -> bool {
    self.enabled
  }

  fn set_enabled(&mut self, value: bool) {
    self.enabled = value;
  }

  fn parent(&self) -> Option<WidgetArc> {
    Some(self.parent.clone())
  }

  fn set_parent(&mut self, parent: Option<WidgetArc>) {
    assert!(parent.is_some());
    match parent {
      Some(p) => self.parent = p,
      _ => unreachable!(),
    }
  }

  fn children(&self) -> Option<ChildWidgetsArc> {
    None
  }

  fn find_children(&self, _id: usize) -> Option<WidgetArc> {
    None
  }

  fn find_direct_children(&self, _id: usize) -> Option<WidgetArc> {
    None
  }

  fn draw(&self, terminal: &mut Terminal) {
    let abs_rect_min = self.abs_rect().min();
    let pos: U16Pos = coord! {x: abs_rect_min.x as u16, y: abs_rect_min.y as u16};

    let frame = terminal.frame_mut();
    frame.set_cursor(crate::ui::frame::Cursor::new(
      pos,
      self.blinking,
      self.hidden,
      self.style,
    ));
  }
}
