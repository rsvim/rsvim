//! The VIM window.

use crate::cart::IRect;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};
use crate::ui::tree::internal::itree::{Itree, ItreeIter, ItreeIterMut};
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::root::WindowRootContainer;
use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

pub mod content;
pub mod root;

#[derive(Debug, Clone)]
/// The VIM window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct Window {
  base: Itree<WindowValue>,
}

impl Window {
  pub fn new(shape: IRect) -> Self {
    let window_root = WindowRootContainer::new();
    let window_root_id = window_root.id();
    let window_root_node = Inode::new(WindowValue::WindowRootContainer(window_root), shape);

    let mut base = Itree::new(window_root_node);

    let window_content = WindowContent::new();
    let window_content_id = window_content.id();
    let window_content_node = Inode::new(WindowValue::WindowContent(window_content), shape);

    base.bounded_insert(&window_root_id, window_content_node);

    Window { base }
  }
}

impl Widget for Window {
  fn id(&self) -> WidgetId {
    self.base.root_id()
  }
}

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum WindowValue {
  WindowContent(WindowContent),
  WindowRootContainer(WindowRootContainer),
}

impl InodeValue for WindowValue {
  fn id(&self) -> InodeId {
    match self {
      WindowValue::WindowContent(w) => w.id(),
      WindowValue::WindowRootContainer(w) => w.id(),
    }
  }
}
