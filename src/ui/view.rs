//! UI components.

use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

use crate::ui::rect::{IPos, Size, UPos};
use crate::ui::term::Terminal;

pub mod root;

/// A [`View`] is a basic trait for all UI components.
pub trait View {
  /// (Relative) offset based on parent [view](crate::ui::view::View).
  fn offset(&self) -> IPos;

  /// Absolute offset based on [terminal](crate::ui::term::Terminal).
  fn abs_offset(&self) -> UPos;

  /// Rectangle height/width.
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

  /// Draw the view to terminal.
  fn draw(&self, terminal: &Terminal);
}

/// The `Rc/RefCell` smart pointer for a view.
pub type ViewRc = Rc<RefCell<dyn View>>;
/// The `Weak/RefCell` smart pointer for a view.
pub type ViewWk = Weak<RefCell<dyn View>>;
