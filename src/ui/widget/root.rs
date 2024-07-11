//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::vec::Vec;

use crate::define_widget_helpers;
use crate::geom::{IRect, URect};
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetArc, WidgetRc, WidgetsArc, WidgetsRc};
use crate::uuid;
use geo::coord;

/// Root widget.
pub struct RootWidget {
  id: usize,
  rect: IRect,
  abs_rect: URect,
  visible: bool,
  enabled: bool,
  children: WidgetsArc,
}

impl RootWidget {
  pub fn new(rect: URect) -> Self {
    RootWidget {
      id: uuid::next(),
      rect: IRect::new(
        coord! {x:0, y:0},
        coord! {x:rect.width as isize, y:rect.height as isize},
      ),
      abs_rect: URect::new(coord! {x:0, y:0}, coord! {x:rect.width , y:rect.height }),
      visible: true,
      enabled: true,
      children: Arc::new(RwLock::new(vec![])),
    }
  }

  define_widget_helpers!();
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

  fn absolute_rect(&self) -> URect {
    self.abs_rect
  }

  /// Not allow to modify the position & size.
  fn set_absolute_rect(&mut self, _rect: URect) {
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

  fn parent(&self) -> Option<WidgetArc> {
    None
  }

  fn set_parent(&mut self, _parent: Option<WidgetArc>) {
    unimplemented!();
  }

  fn children(&self) -> Option<WidgetsArc> {
    Some(self.children.clone())
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

  fn draw(&self, _terminal: &mut Terminal) {
    todo!();
  }
}
