//! Cursor widget.
//! Note: To avoid naming confliction with `crossterm::cursor`, here name it `cursive`.

use crate::geo::pos::U16Pos;
use crate::geo::size::Size;
use crate::ui::widget::{ChildWidgetsRw, Widget, WidgetRw};
use crate::uuid;

pub struct Cursive {
  id: usize,
  pos: U16Pos,
}

impl Cursive {
  pub fn new(pos: U16Pos) -> Self {
    Cursive {
      id: uuid::next(),
      pos,
    }
  }
}

impl Widget for Cursive {}
