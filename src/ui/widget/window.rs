//! The Vim window.

use crate::cart::{self, IRect, U16Size, URect, USize};
use crate::ui::term::Terminal;
use crate::ui::tree::{NodeId, Tree};
use crate::ui::widget::{Widget, WidgetBase};

/// The Vim window.
pub struct Window<'a> {
  base: WidgetBase<'a>,
}

impl<'a> Window<'a> {
  pub fn new(
    parent_id: NodeId,
    tree: &'a mut Tree,
    terminal: &'a mut Terminal,
    rect: IRect,
    zindex: usize,
  ) -> Self {
    let zindex = std::usize::MAX;
    let base = WidgetBase::new(tree, terminal, Some(parent_id), rect, zindex);
    Window { base }
  }
}

impl<'a> Widget<_> for Window<'a> {
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
