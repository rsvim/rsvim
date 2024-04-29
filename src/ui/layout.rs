use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

use crate::ui::view::ViewRc;

pub trait Layout {
  fn children(&self) -> LinkedList<ViewRc>;
}

pub type LayoutRc = Rc<RefCell<dyn Layout>>;
pub type LayoutWk = Weak<RefCell<dyn Layout>>;

pub struct HorizontalLayout {}

impl Layout for HorizontalLayout {
  fn children(&self) -> LinkedList<ViewRc> {
    LinkedList::new()
  }
}

pub struct VerticalLayout {}

impl Layout for VerticalLayout {
  fn children(&self) -> LinkedList<ViewRc> {
    LinkedList::new()
  }
}
