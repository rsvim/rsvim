use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

use crate::ui::device::Device;
use crate::ui::rect::{AbsPos, RelPos, Size};

pub trait View {
  fn draw(&self, dvc: &Device);

  fn offset(&self) -> RelPos;

  fn size(&self) -> Size;

  fn abs_offset(&self) -> AbsPos;

  fn parent(&self) -> Option<Rc<RefCell<dyn View>>>;

  fn children(&self) -> LinkedList<Rc<RefCell<dyn View>>>;
}
