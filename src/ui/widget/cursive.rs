//! Cursor widget.
//! Note: To avoid naming confliction with `crossterm::cursor`, here name it `cursive`.

use crate::geo::size::Size;
use crate::ui::widget::{ChildWidgetsRw, Widget, WidgetRw};
use crate::uuid;

pub struct Cursive {
  id: usize,
}

impl Cursive {
  pub fn new() -> Self {
    Cursive { id: uuid::next() }
  }
}

impl Widget for Cursive {}
