//! Root widget is the root UI container for all other widgets.
//! It always exists along with RSVIM, as long as it runs in non-headless and interactive
//! (non-batch-processing) mode.

use std::any::Any;
use std::cell::RefCell;
use std::intrinsics::unreachable;
use std::rc::{Rc, Weak};
use std::sync::{Arc, RwLock};

use crate::cart::{self, IRect, Size, U16Size, UPos, URect, USize};
use crate::ui::term::Terminal;
use crate::ui::tree::{NodeId, Tree};
use crate::ui::widget::{Widget, WidgetArc, WidgetKind, WidgetRc, WidgetsArc};
use crate::uuid;
use crate::{geo_rect_as, geo_size_as};
use geo::{point, Rect};

/// Root widget.
pub struct RootWidget {
  id: NodeId,
  tree: Weak<Tree>,
  terminal: Weak<Terminal>,

  rect: IRect,

  absolute_rect: URect,        // Cached absolute rect
  actual_rect: IRect,          // Cached actual rect
  actual_absolute_rect: URect, // Cached actual, absolute rect

  visible: bool,
  enabled: bool,
}

impl RootWidget {
  pub fn new(tree: Weak<Tree>, terminal: Weak<Terminal>) -> Self {
    if let Some(t) = terminal.upgrade() {
      let terminal_size = t.size();
      let rect = IRect::new(
        (0, 0),
        (
          terminal_size.width() as isize,
          terminal_size.height() as isize,
        ),
      );
      let urect = geo_rect_as!(rect, usize);
      return RootWidget {
        id: uuid::next(),
        tree,
        terminal,
        rect,
        absolute_rect: urect,
        actual_rect: rect,
        actual_absolute_rect: urect,
        visible: true,
        enabled: true,
      };
    }
    unreachable!("Terminal is None");
  }
}

impl Widget for RootWidget {
  fn id(&self) -> NodeId {
    self.id
  }

  fn tree(&self) -> Weak<Tree> {
    self.tree
  }

  fn terminal(&self) -> Weak<Terminal> {
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

  fn draw(&self) {
    todo!();
  }
}
