//! Utils for UI tree.

use crate::ui::tree::{Tree, TreeNodeId};

use std::ptr::NonNull;

#[derive(Debug, Clone)]
/// Safe wrapper on `NonNull<Tree>`.
///
/// WARNING: Use it only inside the UI tree, when we need to access the `Tree` structure to get
/// some global variables. All UI widgets or tree nodes are managed inside the UI tree, and the UI
/// tree itself is wrapped with the `Arc<RwLock<>>` so it's safe to do so.
/// But once any widget/node is been removed from the UI tree, this reference is no longer safe.
pub struct SafeTreeRef(NonNull<Tree>);

unsafe impl Send for SafeTreeRef {}

unsafe impl Sync for SafeTreeRef {}

impl SafeTreeRef {
  pub fn new(tree: &mut Tree) -> Self {
    SafeTreeRef(NonNull::new(tree as *mut Tree).unwrap())
  }

  /// Ensure the tree reference (held by this struct) still contains the specific tree node.
  ///
  /// # Panics
  /// If the specific tree node (`id`) doesn't belong to the UI tree.
  ///
  /// # Safety
  unsafe fn ensure_has_node(&self, id: &TreeNodeId) {
    self.0.as_ref().node(id).unwrap();
  }

  /// Get `Tree` immutable reference.
  pub fn as_ref(&self, id: &TreeNodeId) -> &Tree {
    unsafe {
      self.ensure_has_node(id);
      self.0.as_ref()
    }
  }

  /// Get `Tree` mutable reference.
  pub fn as_mut(&mut self, id: &TreeNodeId) -> &mut Tree {
    unsafe {
      self.ensure_has_node(id);
      self.0.as_mut()
    }
  }
}
