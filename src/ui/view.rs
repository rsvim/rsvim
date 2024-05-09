use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::ui::canvas::Canvas;
use crate::ui::layout::LayoutRc;
use crate::ui::rect::{IPos, Size, UPos};

/// View
pub trait View {
  /// (Relative) x-y offset vased on parent view
  fn offset(&self) -> IPos;

  /// Absoluate x/y offset based on terminal screen
  fn abs_offset(&self) -> UPos;

  /// Rectangle height/width
  fn size(&self) -> Size;

  /// Control arrange content layout when multiple views conflict on each other.
  fn zindex(&self) -> usize;

  /// Parent view of this view.
  /// Note: Root view doesn't have a parent view.
  fn parent(&self) -> Option<ViewWk>;

  /// Manage children views layout inside this view when there exists.
  fn layout(&self) -> Option<LayoutRc>;

  /// Draw the view to canvas buffer.
  ///
  /// * `canvas`: Canvas buffer
  fn draw(&self, canvas: &Canvas);
}

pub type ViewRc = Rc<RefCell<dyn View>>;
pub type ViewWk = Weak<RefCell<dyn View>>;
