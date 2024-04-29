use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

use crate::ui::view::View;

pub trait Layout {
  fn children(&self) -> LinkedList<Rc<RefCell<dyn View>>>;
}
