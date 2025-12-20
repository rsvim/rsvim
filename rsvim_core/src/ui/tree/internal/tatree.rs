//! Internal tree structure that implements the widget tree.

use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::inode::next_node_id;
use itertools::Itertools;
use std::fmt::Debug;
use std::iter::Iterator;
use taffy::AvailableSpace;
use taffy::Layout;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;

#[derive(Debug, Clone)]
pub struct TaTree {
  ta: TaffyTree,
  // Maps TreeNodeId <==> taffy::NodeId.
  id2taid: FoldMap<TreeNodeId, taffy::NodeId>,
  taid2id: FoldMap<taffy::NodeId, TreeNodeId>,
}

impl TaTree {
  pub fn new() -> Self {
    Self {
      ta: TaffyTree::new(),
      id2taid: FoldMap::new(),
      taid2id: FoldMap::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.id2taid.is_empty()
  }

  pub fn len(&self) -> usize {
    self.id2taid.len()
  }

  #[cfg(not(test))]
  fn _internal_check(&self) {}

  #[cfg(test)]
  fn _internal_check(&self) {
    debug_assert_eq!(self.ta.total_node_count(), self.id2taid.len());
    debug_assert_eq!(self.ta.total_node_count(), self.taid2id.len());

    for (id, taid) in self.id2taid.iter() {
      debug_assert!(self.taid2id.contains_key(taid));
      debug_assert_eq!(*self.taid2id.get(taid).unwrap(), *id);
      if let Some(parent_taid) = self.ta.parent(*taid) {
        debug_assert!(self.taid2id.contains_key(&parent_taid));
      }
      if let Ok(children_taids) = self.ta.children(*taid) {
        for child_taid in children_taids {
          debug_assert!(self.taid2id.contains_key(&child_taid));
        }
      }
    }
    for (taid, nid) in self.taid2id.iter() {
      debug_assert!(self.id2taid.contains_key(nid));
      debug_assert_eq!(*self.id2taid.get(nid).unwrap(), *taid);
    }
  }

  pub fn new_leaf(&mut self, style: Style) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let taid = self.ta.new_leaf(style)?;
    let id = next_node_id();
    self.id2taid.insert(id, taid);
    self.taid2id.insert(taid, id);
    self._internal_check();
    Ok(id)
  }

  pub fn compute_layout(
    &mut self,
    id: TreeNodeId,
    available_size: taffy::Size<AvailableSpace>,
  ) -> TaffyResult<()> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    let result = self.ta.compute_layout(*taid, available_size);
    self._internal_check();
    result
  }

  pub fn layout(&self, id: TreeNodeId) -> TaffyResult<&Layout> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    self.ta.layout(*taid)
  }

  pub fn style(&self, id: TreeNodeId) -> TaffyResult<&Style> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    self.ta.style(*taid)
  }

  pub fn set_style(&mut self, id: TreeNodeId, style: Style) -> TaffyResult<()> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    self.ta.set_style(*taid, style)
  }

  pub fn contains(&self, id: TreeNodeId) -> bool {
    self.id2taid.contains_key(&id)
  }

  pub fn parent(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self._internal_check();
    let taid = self.id2taid.get(&id)?;
    let parent_taid = self.ta.parent(*taid)?;
    self.taid2id.get(&parent_taid).copied()
  }

  pub fn children(&self, id: TreeNodeId) -> TaffyResult<Vec<TreeNodeId>> {
    self._internal_check();
    let taid = self.id2taid.get(&id).unwrap();
    let children_taids = self.ta.children(*taid)?;
    Ok(
      children_taids
        .iter()
        .map(|i| *self.taid2id.get(i).unwrap())
        .collect_vec(),
    )
  }

  pub fn add_child(
    &mut self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
  ) -> TaffyResult<()> {
    self._internal_check();
    let parent_taid = self.id2taid.get(&parent_id).unwrap();
    let child_taid = self.id2taid.get(&child_id).unwrap();
    let result = self.ta.add_child(*parent_taid, *child_taid)?;
    Ok(result)
  }

  pub fn remove_child(
    &mut self,
    parent_id: TreeNodeId,
    child_id: TreeNodeId,
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let parent_taid = self.id2taid.get(&parent_id).unwrap();
    let child_taid = self.id2taid.get(&child_id).unwrap();
    let removed_taid = self.ta.remove_child(*parent_taid, *child_taid)?;
    debug_assert_eq!(removed_taid, *child_taid);
    let removed_id = *self.taid2id.get(&removed_taid).unwrap();
    debug_assert_eq!(removed_id, child_id);
    Ok(removed_id)
  }

  pub fn set_children(
    &mut self,
    parent_id: TreeNodeId,
    children: &[TreeNodeId],
  ) -> TaffyResult<()> {
    self._internal_check();
    let parent_taid = self.id2taid.get(&parent_id).unwrap();
    let children_taids = children
      .iter()
      .map(|i| *self.id2taid.get(i).unwrap())
      .collect_vec();
    self.ta.set_children(*parent_taid, &children_taids)
  }

  pub fn new_with_parent(
    &mut self,
    style: Style,
    parent_id: TreeNodeId,
  ) -> TaffyResult<TreeNodeId> {
    let id = self.new_leaf(style)?;
    self.add_child(parent_id, id)?;
    Ok(id)
  }

  pub fn new_with_children(
    &mut self,
    style: Style,
    children: &[TreeNodeId],
  ) -> TaffyResult<TreeNodeId> {
    self._internal_check();
    let children_taids = children
      .iter()
      .map(|i| *self.id2taid.get(i).unwrap())
      .collect_vec();
    let taid = self.ta.new_with_children(style, &children_taids)?;
    let id = next_node_id();
    self.id2taid.insert(id, taid);
    self.taid2id.insert(taid, id);
    self._internal_check();
    Ok(id)
  }
}
