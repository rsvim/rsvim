//! Cursor widget.

use crate::geo::{IPos, IRect, URect};
use crate::ui::term::Terminal;
use crate::ui::widget::{ChildWidgetsRw, Widget, WidgetRw};
use crate::uuid;
use crossterm::cursor::SetCursorStyle;
use geo::coord;
use std::sync::{Arc, RwLock};

pub struct Cursor {
  parent: WidgetRw,
  id: usize,
  rect: IRect,
  abs_rect: URect,
  visible: bool,
  enabled: bool,

  blinking: bool,
  hidden: bool,
  saved_offset: Option<IPos>, // saved_pos
  style: SetCursorStyle,
}

impl Cursor {
  pub fn new(
    parent: WidgetRw,
    rect: IRect,
    blinking: bool,
    hidden: bool,
    saved_offset: Option<IPos>,
    style: SetCursorStyle,
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
      saved_offset,
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

  fn parent(&self) -> Option<WidgetRw> {
    Some(self.parent.clone())
  }

  fn set_parent(&mut self, parent: Option<WidgetRw>) {
    assert!(parent.is_some());
    match parent {
      Some(p) => self.parent = p,
      _ => unreachable!(),
    }
  }

  fn children(&self) -> ChildWidgetsRw {
    Arc::new(RwLock::new(vec![]))
  }

  fn find_children(&self, _id: usize) -> Option<WidgetRw> {
    None
  }

  fn find_direct_children(&self, _id: usize) -> Option<WidgetRw> {
    None
  }

  fn draw(&self, terminal: &Terminal) {}
}
