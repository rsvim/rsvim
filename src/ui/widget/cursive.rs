//! Cursor widget.
//! Note: To avoid naming confliction with `crossterm::cursor`, here name it `cursive`.

use crate::ui::widget::{ChildWidgetsRw, Widget, WidgetRw};

pub struct Cursive {}

impl Widget for Cursive {}
