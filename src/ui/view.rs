use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

use crate::ui::rect::{IPos, Size, UPos};
use crate::ui::term::Terminal;

pub mod root;

/// View
pub trait View {
  /// (Relative) x/y offset vased on parent view
  fn offset(&self) -> IPos;

  /// Absolute x/y offset based on terminal
  fn abs_offset(&self) -> UPos;

  /// Rectangle height/width
  fn size(&self) -> Size;

  /// Control arrange content layout when multiple views conflict on each other.
  /// View that contains a higher zindex will cover the lower one.
  fn zindex(&self) -> usize;

  /// Parent view of this view.
  ///
  /// Note: Root view doesn't have a parent view.
  fn parent(&self) -> Option<ViewWk>;

  /// Children views of this view.
  ///
  /// Note: View has the **ownership** of all its children, thus recursively **owns** all its nested
  /// grandchildren and so on, which means:
  /// 1. The (grand)children will be destroyed once their parent is been destroyed.
  /// 2. The (grand)children still can be placed outside of their parent, i.e. the size or position
  ///    can be outside the scope of their parent.
  fn children(&self) -> LinkedList<ViewWk>;

  /// Draw the view to canvas buffer.
  fn draw(&self, terminal: &Terminal);
}

pub type ViewRc = Rc<RefCell<dyn View>>;
pub type ViewWk = Weak<RefCell<dyn View>>;
