use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

use crate::ui::view::{View, ViewPtr};

pub trait Layout {
  fn children(&self) -> LinkedList<ViewPtr>;
}

pub type LayoutPtr = Rc<RefCell<dyn Layout>>;

pub struct HorizontalLayout {}

impl Layout for HorizontalLayout {
  fn children(&self) -> LinkedList<Rc<RefCell<dyn View>>> {
    LinkedList::new()
  }
}

pub struct VerticalLayout {}

impl Layout for VerticalLayout {
  fn children(&self) -> LinkedList<Rc<RefCell<dyn View>>> {
    LinkedList::new()
  }
}
