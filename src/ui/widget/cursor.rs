//! Cursor widget.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::vec::Vec;

use crate::geom::{IPos, IRect, U16Pos, UPos, URect};
use crate::ui::frame::CursorStyle;
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetArc, WidgetKind, WidgetRc, WidgetsArc, WidgetsRc};
use crate::uuid;
use crate::{as_geo_rect, as_geo_size, define_widget_helpers};
use geo::point;

pub struct Cursor {
  parent: WidgetArc,
  id: usize,
  pos: IPos,
  visible: bool,
  enabled: bool,

  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(
    parent: WidgetArc,
    pos: IPos,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> Self {
    Cursor {
      parent,
      id: uuid::next(),
      pos,
      visible: true,
      enabled: true,
      blinking,
      hidden,
      style,
    }
  }

  define_widget_helpers!();
}

impl Widget for Cursor {
  fn id(&self) -> usize {
    self.id
  }

  fn kind(&self) -> WidgetKind {
    WidgetKind::CursorKind
  }

  fn rect(&self) -> IRect {
    IRect::new(self.pos, point!(x: self.pos.x() + 1, y: self.pos.y() + 1))
  }

  fn set_rect(&mut self, rect: IRect) {
    self.pos = rect.min().into();
  }

  fn zindex(&self) -> usize {
    std::usize::MAX
  }

  fn set_zindex(&mut self, _zindex: usize) {
    unimplemented!()
  }

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
    match parent {
      Some(p) => self.parent = p,
      _ => unreachable!("Parent is None"),
    }
  }

  fn children(&self) -> Option<WidgetsArc> {
    unreachable!("Cursor doesn't have children widgets")
  }

  fn set_children(&mut self, _children: Option<WidgetsArc>) {
    unreachable!("Cursor cannot set children widgets")
  }

  fn find_children(&self, _id: usize) -> Option<WidgetArc> {
    unreachable!("Cursor doesn't have children widgets")
  }

  fn find_direct_children(&self, _id: usize) -> Option<WidgetArc> {
    unreachable!("Cursor doesn't have direct children widgets")
  }

  fn draw(&self, terminal: &mut Terminal) {
    let abs_rect_min = self.absolute_rect().min();
    let pos: U16Pos = point! (x: abs_rect_min.x as u16, y: abs_rect_min.y as u16);

    let frame = terminal.frame_mut();
    frame.set_cursor(crate::ui::frame::Cursor::new(
      pos,
      self.blinking,
      self.hidden,
      self.style,
    ));
  }
}
