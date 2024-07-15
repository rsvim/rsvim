//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, RwLock};

use crate::cart::{self, IRect, Size, U16Size, UPos, URect, USize};
use crate::ui::term::Terminal;
use crate::ui::tree::{NodeId, Tree};
use crate::ui::widget::{Widget, WidgetBase};
use crate::uuid;
use crate::{geo_rect_as, geo_size_as};
use geo::{point, Rect};

/// Root widget.
pub struct RootWidget<'a> {
  base: WidgetBase<'a>,
}

impl<'a> RootWidget<'a> {
  pub fn new(tree: &'a mut Tree, terminal: &'a mut Terminal) -> Self {
    let terminal_size = terminal.size();
    let rect = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    let zindex = 0;
    let base = WidgetBase::new(tree, terminal, None, rect, zindex);
    RootWidget { base }
  }
}

impl<'a> Widget<_> for RootWidget<'a> {
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
    todo!();
  }
}
