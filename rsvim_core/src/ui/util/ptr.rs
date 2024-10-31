//! Pointers for UI tree.
//!
//! WARNING: These pointers are only allowed to be used inside the UI tree and widgets. The
//! scenario is: when some widgets want to access the data located in the tree or other nodes, the
//! widgets will have to contains a pointer/reference to the tree.
//! This is actually safe because all the widget nodes are managed by the tree. The only dangerous
//! case is: a widget is been removed from the tree, thus the tree pointer/reference held by this
//! widget is no longer valid.

use crate::ui::tree::{Tree, TreeNodeId};
use crate::ui::widget::window::{Viewport, Window};

use std::convert::{AsMut, AsRef};
use std::ptr::NonNull;

#[derive(Debug, Clone)]
/// Safe wrapper on [`NonNull<Tree>`](Tree).
pub struct SafeTreeRef(NonNull<Tree>, TreeNodeId);

unsafe impl Send for SafeTreeRef {}

unsafe impl Sync for SafeTreeRef {}

impl SafeTreeRef {
  pub fn new(tree: &mut Tree, self_id: TreeNodeId) -> Self {
    SafeTreeRef(NonNull::new(tree as *mut Tree).unwrap(), self_id)
  }

  /// Ensure the tree reference (held by **the** struct) still contains **the** node.
  ///
  /// # Panics
  /// If **the** tree node (`id`) doesn't belong to the UI tree.
  ///
  /// # Safety
  unsafe fn ensure_has_node(&self, id: &TreeNodeId) {
    self.0.as_ref().node(id).unwrap();
  }
}

impl AsRef<Tree> for SafeTreeRef {
  fn as_ref(&self) -> &Tree {
    unsafe {
      self.ensure_has_node(&self.1);
      self.0.as_ref()
    }
  }
}

impl AsMut<Tree> for SafeTreeRef {
  fn as_mut(&mut self) -> &mut Tree {
    unsafe {
      self.ensure_has_node(&self.1);
      self.0.as_mut()
    }
  }
}

#[derive(Debug, Clone)]
/// Safe wrapper on [`NonNull<Window>`](Window).
pub struct SafeWindowRef(NonNull<Window>);

unsafe impl Send for SafeWindowRef {}

unsafe impl Sync for SafeWindowRef {}

impl SafeWindowRef {
  pub fn new(window: &mut Window) -> Self {
    SafeWindowRef(NonNull::new(window as *mut Window).unwrap())
  }
}

impl AsRef<Window> for SafeWindowRef {
  fn as_ref(&self) -> &Window {
    unsafe { self.0.as_ref() }
  }
}

impl AsMut<Window> for SafeWindowRef {
  fn as_mut(&mut self) -> &mut Window {
    unsafe { self.0.as_mut() }
  }
}

#[derive(Debug, Clone)]
/// Safe wrapper on [`NonNull<Viewport>`](Viewport).
pub struct SafeViewportRef(NonNull<Viewport>);

unsafe impl Send for SafeViewportRef {}

unsafe impl Sync for SafeViewportRef {}

impl SafeViewportRef {
  pub fn new(viewport: &mut Viewport) -> Self {
    SafeViewportRef(NonNull::new(viewport as *mut Viewport).unwrap())
  }
}

impl AsRef<Viewport> for SafeViewportRef {
  fn as_ref(&self) -> &Viewport {
    unsafe { self.0.as_ref() }
  }
}

impl AsMut<Viewport> for SafeViewportRef {
  fn as_mut(&mut self) -> &mut Viewport {
    unsafe { self.0.as_mut() }
  }
}
