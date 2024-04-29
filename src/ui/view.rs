use std::rc::Rc;

use crate::ui::device::Device;
use crate::ui::layout::LayoutRc;
use crate::ui::rect::{AbsPos, RelPos, Size};

pub trait View {
  fn draw(&self, dvc: &Device);

  fn offset(&self) -> RelPos;

  fn size(&self) -> Size;

  fn abs_offset(&self) -> AbsPos;

  fn parent(&self) -> Option<ViewRc>;

  fn layout(&self) -> LayoutRc;
}

pub type ViewRc = Rc<dyn View>;
