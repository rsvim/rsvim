use crate::prelude::*;
use crate::ui::tree::internal::context::*;
use crate::ui::tree::internal::inode::*;
use crate::ui::tree::internal::shapes;
use itertools::Itertools;
use std::collections::VecDeque;
use taffy::TaffyResult;

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

  pub fn children_ids(&self, id: TreeNodeId) -> TaffyResult<Vec<TreeNodeId>> {
    self._internal_check();
    self.context.borrow().children(id)
  }

  /// Get nodes.
  pub fn nodes(&self) -> &FoldMap<TreeNodeId, T> {
    self._internal_check();
    &self.nodes
  }

  /// Get mutable nodes.
  pub fn nodes_mut(&mut self) -> &mut FoldMap<TreeNodeId, T> {
    self._internal_check();
    &mut self.nodes
  }

  pub fn iter(&self) -> ItreeIter<'_, T> {
    ItreeIter::new(self, Some(self.root_id()))
  }
}

impl<T> Itree<T>
where
  T: Inodeable,
{
  pub fn raw_move_position_by(
    &self,
    context: &TreeContext,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> IRect {
    let shape = context.shape(id).unwrap();
    let pos: IPos = shape.min().into();
    self.raw_move_position_to(context, id, pos.x() + x, pos.y() + y)
  }

  pub fn raw_move_position_to(
    &self,
    context: &TreeContext,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> IRect {
    let shape = context.shape(id).unwrap();
    let new_pos = point!(x, y);
    rect!(
      new_pos.x(),
      new_pos.y(),
      new_pos.x() + shape.width(),
      new_pos.y() + shape.height()
    )
  }

  /// Calculates a widget shape by relative motion on its parent:
  /// - It moves to left when `x < 0`.
  /// - It moves to right when `x > 0`.
  /// - It moves to up when `y < 0`.
  /// - It moves to down when `y > 0`.
  ///
  /// This motion uses the [TruncatePolicy::RESERVED] like policy, e.g. if it
  /// hits the boundary of its parent, it simply stops moving to avoid its size
  /// been truncated by its parent.
  ///
  /// Returns the new shape after movement if successfully, otherwise
  /// returns `None` if the node doesn't exist or doesn't have a parent.
  pub fn reserved_move_position_by(
    &self,
    context: &TreeContext,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> IRect {
    let parent_id = context.parent(id)?;
    let shape = context.shape(id)?;
    let pos: IPos = shape.min().into();
    let new_pos = point!(pos.x() + x, pos.y() + y);
    let new_shape = rect!(
      new_pos.x(),
      new_pos.y(),
      new_pos.x() + shape.width(),
      new_pos.y() + shape.height()
    );
    let parent_actual_shape = context.actual_shape(parent_id)?;
    let final_shape =
      shapes::bound_shape(&new_shape, &parent_actual_shape.size());
    let final_pos: IPos = final_shape.min().into();
    let final_x = final_pos.x() - pos.x();
    let final_y = final_pos.y() - pos.y();
    self.raw_move_position_by(&context, id, final_x, final_y)
  }

  /// Similar to [reserved_move_position_by](Self::reserved_move_position_by),
  /// but moves with absolute position instead of relative.
  pub fn reserved_move_position_to(
    &self,
    context: &TreeContext,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> IRect {
    let parent_id = context.parent(id)?;
    let shape = context.shape(id).unwrap();
    let new_pos: IPos = point!(x, y);
    let new_shape = rect!(
      new_pos.x(),
      new_pos.y(),
      new_pos.x() + shape.width(),
      new_pos.y() + shape.height()
    );
    let parent_actual_shape = context.actual_shape(parent_id)?;
    let final_shape =
      shapes::bound_shape(&new_shape, &parent_actual_shape.size());
    let final_pos: IPos = final_shape.min().into();
    self.raw_move_position_to(&context, id, final_pos.x(), final_pos.y())
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
        if self.tree.nodes().get(&child_id).is_some() {
          self.que.push_back(child_id);
        }
      }
      self.tree.nodes.get(&id)
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
