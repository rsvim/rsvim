//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::geom::{self, IRect, Size, U16Size, UPos, URect, USize};
use crate::ui::term::Terminal;
use crate::ui::widget::{Widget, WidgetArc, WidgetKind, WidgetRc, WidgetsArc};
use crate::uuid;
use crate::{as_geo_rect, as_geo_size, define_widget_helpers};
use geo::{point, Rect};

/// Root widget.
pub struct RootWidget {
  id: usize,
  terminal: TerminalArc,

  rect: IRect,

  absolute_rect: URect,        // Cached absolute rect
  actual_rect: IRect,          // Cached actual rect
  actual_absolute_rect: URect, // Cached actual, absolute rect

  visible: bool,
  enabled: bool,
  children: WidgetsArc,
}

impl RootWidget {
  pub fn new(terminal: TerminalArc) -> Self {
    let terminal_size = terminal.frame().size;
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
      terminal,
      rect,
      absolute_rect: urect,
      actual_rect: rect,
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

  fn terminal(&self) -> TerminalArc {
    self.terminal
  }

  fn rect(&self) -> IRect {
    self.rect
  }

  fn set_rect(&mut self, rect: IRect) {
    self.rect = rect;
    let absolute_rect = self.to_absolute_rect();
    self._set_absolute_rect();
  }

  fn absolute_rect(&self) -> IRect {
    self.absolute_rect
  }

  fn set_absolute_rect(&mut self, rect: URect) {
    self.absolute_rect = rect;
  }

  /// Get actual rect. i.e. relative position and actual size.
  fn actual_rect(&self) -> IRect;

  /// Set/cache actual rect.
  fn set_actual_rect(&mut self, rect: IRect);

  /// Get actual absolute rect. i.e. absolute position and actual size.
  fn actual_absolute_rect(&self) -> URect;

  /// Set/cache actual absolute rect.
  fn set_actual_absolute_rect(&mut self, rect: URect);

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
