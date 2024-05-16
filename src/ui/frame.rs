//! UI components: [frame](Frame), window, floatwindow, tabline, statusline, numbercolumn, etc.

use crate::ui::geo::pos::{IPos, UPos};
use crate::ui::geo::size::Size;
use crate::ui::term::Terminal;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::{Rc, Weak};

pub mod root;

/// A frame is a basic container for all UI components, it specifies the basics of a UI component: position, size.
pub trait Frame {
  /// (Relative) offset based on parent frame.
  fn offset(&self) -> IPos;

  /// Absolute offset based on whole [terminal](crate::ui::term::Terminal).
  fn abs_offset(&self) -> UPos;

  /// Frame size.
  fn size(&self) -> Size;

  /// Control arrange content layout when multiple views conflict on each other.
  /// A frame that has a higher zindex will cover/override the lower one.
  fn zindex(&self) -> usize;

  /// Parent frame of this one.
  ///
  /// Note: Root frame doesn't have a parent.
  fn parent(&self) -> Option<FrameWk>;

  /// Children frames of this one.
  ///
  /// Note: View **owns** all its children, thus recursively **owns** all its nested
  /// grandchildren and so on, which means:
  /// 1. The (grand)children will be destroyed once their parent is been destroyed.
  /// 2. The (grand)children can only be *logically* placed outside of their parent, but the outside
  ///    parts will be invisible, only those parts inside their parent's rectangle are visible.
  fn children(&self) -> LinkedList<FrameWk>;

  /// Draw the view to terminal.
  fn draw(&self, terminal: &Terminal);
}

/// The `Rc/RefCell` smart pointer for a [frame](Frame).
pub type FrameRc = Rc<RefCell<dyn Frame>>;
/// The `Weak/RefCell` smart pointer for a [frame](Frame).
pub type FrameWk = Weak<RefCell<dyn Frame>>;
