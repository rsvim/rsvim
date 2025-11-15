//! Internal tree.

use crate::ui::tree::Tree;
use crate::ui::tree::TreeNode;
use crate::ui::tree::internal::LayoutNodeId;
use std::collections::VecDeque;

#[derive(Debug)]
/// The iterator of the tree, it traverse the tree from the root node in
/// level-order. This helps us render the whole UI tree, because the root node
/// is at the bottom of canvas, leaf nodes are at the top of canvas.
pub struct TreeIter<'a> {
  tree: &'a Tree,
  que: VecDeque<LayoutNodeId>,
}

impl<'a> Iterator for TreeIter<'a> {
  type Item = &'a TreeNode;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(layout_id) = self.que.pop_front() {
      if let Ok(children_layout_ids) =
        self.tree.layout_tree.borrow().children(layout_id)
      {
        for child_layout_id in children_layout_ids {
          if self.tree.layout2nodeids.contains_key(&child_layout_id) {
            self.que.push_back(child_layout_id);
          }
        }
      }
      let node_id = self.tree.layout2nodeids.get(&layout_id).unwrap();
      self.tree.node(*node_id)
    } else {
      None
    }
  }
}

impl<'a> TreeIter<'a> {
  pub fn new(tree: &'a Tree, start_layout_id: Option<LayoutNodeId>) -> Self {
    let mut que = VecDeque::new();
    if let Some(start_layout_id) = start_layout_id {
      que.push_back(start_layout_id);
    }
    Self { tree, que }
  }
}
