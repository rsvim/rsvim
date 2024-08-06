//! Basic atom of all UI components.

use tracing::debug;

use std::io::Result as IoResult;

use crate::cart::U16Rect;
use crate::ui::term::TerminalArc;
use crate::ui::tree::internal::inode::{InodeId, InodeValue};

// Re-export
pub use crate::ui::widget::container::root::RootContainer;
pub use crate::ui::widget::container::window::WindowContainer;
pub use crate::ui::widget::cursor::Cursor;
pub use crate::ui::widget::window::content::WindowContent;

pub mod container;
pub mod cursor;
pub mod window;

pub type WidgetId = usize;

/// Widget is the base trait for all UI components, it provide a common layer for rendering.
pub trait Widget {
  fn id(&self) -> WidgetId;

  /// Draw the widget to terminal, on the specific shape.
  fn draw(
    &mut self,
    actual_shape: U16Rect,
    _terminal: TerminalArc,
  ) -> impl std::future::Future<Output = IoResult<()>> + Send {
    async move {
      // Do nothing.
      debug!("draw, actual shape:{:?}", actual_shape);
      Ok(())
    }
  }
}

#[derive(Debug, Clone)]
pub enum WidgetValue {
  RootContainer(RootContainer),
  WindowContainer(WindowContainer),
  WindowContent(WindowContent),
  Cursor(Cursor),
}

impl InodeValue for WidgetValue {
  fn id(&self) -> InodeId {
    Widget::id(self)
  }
}

impl Widget for WidgetValue {
  fn id(&self) -> InodeId {
    match self {
      WidgetValue::RootContainer(w) => w.id(),
      WidgetValue::WindowContainer(w) => w.id(),
      WidgetValue::WindowContent(w) => w.id(),
      WidgetValue::Cursor(w) => w.id(),
    }
  }

  async fn draw(&mut self, actual_shape: U16Rect, terminal: TerminalArc) -> IoResult<()> {
    match self {
      WidgetValue::RootContainer(w) => w.draw(actual_shape, terminal).await,
      WidgetValue::WindowContainer(w) => w.draw(actual_shape, terminal).await,
      WidgetValue::WindowContent(w) => w.draw(actual_shape, terminal).await,
      WidgetValue::Cursor(w) => w.draw(actual_shape, terminal).await,
    }
  }
}
