//! Cursor widget.

use crate::cart::{U16Pos, U16Rect};
use crate::ui::frame::CursorStyle;
use crate::ui::term::TerminalWk;
use crate::ui::tree::node::NodeId;
use crate::ui::widget::Widget;
use crate::uuid;

pub struct Cursor {
  id: NodeId,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(blinking: bool, hidden: bool, style: CursorStyle) -> Self {
    Cursor {
      id: uuid::next(),
      blinking,
      hidden,
      style,
    }
  }
}

impl Default for Cursor {
  fn default() -> Self {
    Cursor {
      id: uuid::next(),
      blinking: true,
      hidden: false,
      style: CursorStyle::DefaultUserShape,
    }
  }
}

impl Widget for Cursor {
  fn id(&self) -> NodeId {
    self.id
  }

  fn draw(&mut self, actual_shape: &U16Rect, terminal: TerminalWk) {
    let pos: U16Pos = actual_shape.min().into();

    terminal
      .upgrade()
      .unwrap()
      .write()
      .unwrap()
      .frame_mut()
      .set_cursor(crate::ui::frame::Cursor::new(
        pos,
        self.blinking,
        self.hidden,
        self.style,
      ));
  }
}
