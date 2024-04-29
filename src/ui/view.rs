use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::ui::device::Device;
use crate::ui::layout::LayoutRc;
use crate::ui::rect::{AbsPos, RelPos, Size};

pub trait View {
  // relative offset based on parent view
  fn offset(&self) -> RelPos;

  // absolute offset based on terminal screen
  fn abs_offset(&self) -> AbsPos;

  // height/width
  fn size(&self) -> Size;

  // parent view, root view doesn't have parent
  fn parent(&self) -> Option<ViewWk>;

  // if contains more children views, all these views are managed by layout
  fn layout(&self) -> Option<LayoutRc>;

  // actually draw the terminal screen
  fn draw(&self, dvc: &Device);
}

pub type ViewRc = Rc<RefCell<dyn View>>;
pub type ViewWk = Weak<RefCell<dyn View>>;
