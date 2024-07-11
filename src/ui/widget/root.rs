//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::geom::{IRect, U16Size, URect};
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetArc, WidgetKind, WidgetRc, WidgetsArc, WidgetsRc};
use crate::uuid;
use crate::{as_geo_rect, define_widget_helpers};
use geo::{point, Rect};
use std::any::Any;

/// Root widget.
pub struct RootWidget {
  id: usize,
  rect: IRect,

  absolute_rect: IRect,        // Cached absolute rect
  actual_absolute_rect: URect, // Cached actual, absolute rect

  visible: bool,
  enabled: bool,
  children: WidgetsArc,
}

impl RootWidget {
  pub fn new(terminal_size: U16Size) -> Self {
    let rect = IRect::new(
      (0, 0),
      (
        terminal_size.width() as isize,
        terminal_size.height() as isize,
      ),
    );
    let urect = as_geo_rect!(rect, usize);
    RootWidget {
      id: uuid::next(),
      rect,
      absolute_rect: rect,
      actual_absolute_rect: urect,
      visible: true,
      enabled: true,
      children: vec![],
    }
  }

  define_widget_helpers!();
}

impl Widget for RootWidget {
  fn id(&self) -> usize {
    self.id
  }

  fn kind(&self) -> WidgetKind {
    WidgetKind::RootWidgetKind
  }

  fn rect(&self) -> IRect {
    self.rect
  }

  /// Not allow to modify the position & size.
  fn set_rect(&mut self, rect: IRect) {
    self.rect = rect;
  }

  fn absolute_rect(&self) -> URect {
    self.absolute_rect
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
