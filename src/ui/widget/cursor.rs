//! Cursor widget.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::vec::Vec;

use crate::cart::{IPos, IRect, U16Pos, UPos, URect};
use crate::ui::frame::CursorStyle;
use crate::ui::term::Terminal;
use crate::ui::tree::{NodeId, Tree};
use crate::ui::widget::{Widget, WidgetBase};
use crate::uuid;
use crate::{geo_rect_as, geo_size_as};
use geo::point;

pub struct Cursor<'a> {
  base: WidgetBase<'a>,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl<'a> Cursor<'a> {
  pub fn new(
    parent_id: NodeId,
    tree: &'a mut Tree,
    terminal: &'a mut Terminal,
    pos: IPos,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> Self {
    let rect = IRect::new(pos, pos + point!(x:1, y:1));
    let zindex = std::usize::MAX;
    let base = WidgetBase::new(tree, terminal, Some(parent_id), rect, zindex);
    Cursor {
      base,
      blinking,
      hidden,
      style,
    }
  }
}

impl<'a> Widget<_> for Cursor<'a> {
  fn id(&self) -> NodeId {
    self.base.id()
  }

  fn parent_id(&self) -> Option<NodeId> {
    self.base.parent_id()
  }

  fn tree(&mut self) -> &mut Tree {
    self.base.tree()
  }

  fn terminal(&mut self) -> &mut Terminal {
    self.base.terminal()
  }

  fn rect(&self) -> IRect {
    self.base.rect()
  }

  fn set_rect(&mut self, rect: IRect) {
    self.base.set_rect(rect);
  }

  fn absolute_rect(&self) -> URect {
    self.base.absolute_rect()
  }

  fn actual_rect(&self) -> IRect {
    self.base.actual_rect()
  }

  fn actual_absolute_rect(&self) -> URect {
    self.base.actual_absolute_rect()
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
