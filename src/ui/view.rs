use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;

use crate::ui::device::Device;
use crate::ui::vec::pos::{AbsPos, RelPos};
use crate::ui::vec::vec2d::AbsVec2d;

pub trait View {
  fn draw(&self, dvc: &Device);

  fn offset(&self) -> AbsPos;

  fn size(&self) -> AbsVec2d;

  fn relative_offset(&self) -> RelPos;

  fn parent(&self) -> Option<Rc<RefCell<dyn View>>>;

  fn children(&self) -> LinkedList<Rc<RefCell<dyn View>>>;
}
