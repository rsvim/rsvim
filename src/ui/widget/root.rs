//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use crate::geo::{IRect, URect, USize};
use crate::ui::term::Terminal;
use crate::ui::widget::{ChildWidgetsRw, Widget, WidgetRw};
use crate::uuid;
use geo::coord;
use std::sync::{Arc, RwLock};

/// Root widget.
pub struct RootWidget {
  id: usize,
  rect: IRect,
  abs_rect: URect,
  visible: bool,
  enabled: bool,
  children: ChildWidgetsRw,
}

impl RootWidget {
  pub fn new(size: USize) -> Self {
    RootWidget {
      id: uuid::next(),
      rect: IRect::new(
        coord! {x:0, y:0},
        coord! {x:size.width as isize, y:size.height as isize},
      ),
      abs_rect: URect::new(coord! {x:0, y:0}, coord! {x:size.width , y:size.height }),
      visible: true,
      enabled: true,
      children: Arc::new(RwLock::new(vec![])),
    }
  }
}

impl Widget for RootWidget {
  fn id(&self) -> usize {
    self.id
  }

  fn rect(&self) -> IRect {
    self.rect
  }

  /// Not allow to modify the position & size.
  fn set_rect(&mut self, _rect: IRect) {
    unimplemented!();
  }

  fn abs_rect(&self) -> URect {
    self.abs_rect
  }

  /// Not allow to modify the position & size.
  fn set_abs_rect(&mut self, _rect: URect) {
    unimplemented!();
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
    None
  }

  fn set_parent(&mut self, _parent: Option<WidgetRw>) {
    unimplemented!();
  }

  fn children(&self) -> ChildWidgetsRw {
    self.children.clone()
  }

  fn find_children(&self, _id: usize) -> Option<WidgetRw> {
    None
  }

  fn find_direct_children(&self, _id: usize) -> Option<WidgetRw> {
    None
  }

  fn draw(&self, _terminal: &Terminal) {
    todo!();
  }
}
