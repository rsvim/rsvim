use crate::prelude::*;
use crate::ui::tree::internal::context::*;
use crate::ui::tree::internal::inode::*;
use itertools::Itertools;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use taffy::Style;
use taffy::TaffyResult;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;
use taffy::prelude::TaffyAuto;

#[derive(Debug, Clone)]
pub struct Itree<T>
where
  T: Inodeable,
{
  // The tree context.
  context: TreeContextRc,

  // Node collections.
  nodes: FoldMap<TreeNodeId, T>,
}

impl<T> Itree<T>
where
  T: Inodeable,
{
  pub fn new() -> Self {
    Self {
      context: TreeContext::to_rc(TreeContext::new()),
      nodes: FoldMap::new(),
    }
  }

  fn _internal_check(&self) {
    debug_assert_eq!(self.context.borrow().len(), self.nodes.len());
  }

  pub fn len(&self) -> usize {
    self._internal_check();
    self.nodes.len()
  }

  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.nodes.is_empty()
  }

  pub fn context(&self) -> TreeContextRc {
    self.context.clone()
  }

  pub fn root_id(&self) -> TreeNodeId {
    self._internal_check();
    self.context.borrow().root()
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self._internal_check();
    self.context.borrow().parent(id)
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self._internal_check();
    self.context.borrow().children(id).unwrap_or_default()
  }

  /// Get node by its `id`.
  pub fn node(&self, id: TreeNodeId) -> Option<&T> {
    self._internal_check();
    self.nodes.get(&id)
  }

  /// Get mutable node by its `id`.
  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut T> {
    self._internal_check();
    self.nodes.get_mut(&id)
  }

  pub fn iter(&self) -> ItreeIter<'_, T> {
    ItreeIter::new(self, Some(self.root_id()))
  }
}

// ItreeIter {
#[derive(Debug)]
pub struct ItreeIter<'a, T>
where
  T: Inodeable,
{
  tree: &'a Itree<T>,
  que: VecDeque<TreeNodeId>,
}

impl<'a, T> Iterator for ItreeIter<'a, T>
where
  T: Inodeable,
{
  type Item = &'a T;

  fn next(&mut self) -> Option<Self::Item> {
    if let Some(id) = self.que.pop_front() {
      // Visit all children nodes under a parent node by following Z-index,
      // from higher to lower.
      let children_ids_sorted_by_zindex = {
        let ctx = self.tree.context();
        let ctx = ctx.borrow();
        ctx
          .children(id)
          .unwrap_or_default()
          .iter()
          .sorted_by_key(|i| ctx.zindex(**i).unwrap())
          .rev()
          .copied()
          .collect_vec()
      };
      for child_id in children_ids_sorted_by_zindex {
        if self.tree.node(child_id).is_some() {
          self.que.push_back(child_id);
        }
      }
      self.tree.node(id)
    } else {
      None
    }
  }
}

impl<'a, T> ItreeIter<'a, T>
where
  T: Inodeable,
{
  pub fn new(tree: &'a Itree<T>, start_id: Option<TreeNodeId>) -> Self {
    let mut que = VecDeque::new();
    if let Some(id) = start_id {
      que.push_back(id);
    }
    Self { tree, que }
  }
}
// ItreeIter }
