use crate::geo::pos::{IPos, UPos};
use crate::geo::size::Size;
use crate::id;
use crate::ui::term::Terminal;
use crate::ui::widget::{ChildWidgetsRw, Widget, WidgetRw};
use std::collections::LinkedList;
use std::sync::{Arc, Mutex, RwLock};

pub struct Window {
  id: usize,
  offset: IPos,
  abs_offset: UPos,
  size: Size,
  visible: bool,
  enabled: bool,
  content: ChildWidgetsRw,
}
