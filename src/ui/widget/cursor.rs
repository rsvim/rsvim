//! Cursor widget.

use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::vec::Vec;

use crate::define_widget_helpers;
use crate::geo::{IPos, IRect, U16Pos, UPos, URect};
use crate::ui::frame::CursorStyle;
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetArc, WidgetRc, WidgetsArc, WidgetsRc};
use crate::uuid;
use geo::point;

pub struct Cursor {
  parent: WidgetArc,
  id: usize,
  pos: IPos,
  abs_pos: UPos,
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
    let parent_abs_pos = parent.clone().read().unwrap().absolute_pos();
    let abs_pos: UPos =
      point!(x: pos.x + parent_abs_pos.x as isize, y: pos.y + parent_abs_pos.y as isize);

    Cursor {
      parent,
      id: uuid::next(),
      pos,
      abs_pos: point! (x:0_usize ,y:0_usize),
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

  fn pos(&self) -> IPos {
    self.pos
  }

  fn set_pos(&mut self, pos: IPos) {
    self.pos = pos;
  }

  fn absolute_pos(&self) -> UPos {
    self.abs_pos
  }

  fn set_absolute_pos(&mut self, pos: UPos) {}

  /// Get (logic) size.
  fn size(&self) -> USize;

  /// Set (logic) size.
  fn set_size(&mut self, size: USize);

  /// Get actual size.
  fn actual_size(&self) -> USize;

  /// Set actual size.
  /// If the actual size is out of parent's shape, it will be automatically truncated.
  fn set_actual_size(&mut self, size: USize);

  /// Get (relative) rect.
  /// It indicates both positions and (logic) size.
  fn rect(&self) -> IRect;

  /// Set (relative) rect.
  fn set_rect(&mut self, rect: IRect);

  /// Get absolute rect.
  fn absolute_rect(&self) -> URect;

  /// Set absolute rect.
  fn set_absolute_rect(&mut self, rect: URect);

  /// Get (relative) rect with actual size.
  fn actual_rect(&self) -> IRect;

  /// Set (relative) rect with actual size.
  /// If the actual size is out of parent's shape, it will be automatically truncated.
  fn set_rect(&mut self, rect: IRect);

  /// Get absolute rect with actual size.
  fn actual_absolute_rect(&self) -> URect;

  /// Set absolute rect with actual size.
  /// If the actual size is out of parent's shape, it will be automatically truncated.
  fn set_actual_absolute_rect(&mut self, rect: URect);

  fn rect(&self) -> IRect {
    self.rect
  }

  fn set_rect(&mut self, rect: IRect) {
    self.rect = rect;
  }

  fn absolute_rect(&self) -> URect {
    self.absolute_rect
  }

  fn set_absolute_rect(&mut self, rect: URect) {
    self.absolute_rect = rect;
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

  fn children(&self) -> Option<WidgetsArc> {
    unimplemented!();
  }

  fn set_children(&mut self, _children: Option<WidgetsArc>) {
    unimplemented!();
  }

  fn find_children(&self, _id: usize) -> Option<WidgetArc> {
    unimplemented!();
  }

  fn find_direct_children(&self, _id: usize) -> Option<WidgetArc> {
    unimplemented!();
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
