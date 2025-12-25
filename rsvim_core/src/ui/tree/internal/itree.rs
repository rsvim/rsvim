//! Internal tree structure that implements the widget tree.

use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::internal::context::DEFAULT_TRUNCATE_POLICY;
use crate::ui::tree::internal::context::DEFAULT_ZINDEX;
use crate::ui::tree::internal::context::TreeContext;
use crate::ui::tree::internal::context::TreeContextRc;
use crate::ui::tree::internal::context::TruncatePolicy;
use crate::ui::tree::internal::shapes;
use itertools::Itertools;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::Iterator;
use std::rc::Rc;
use taffy::Style;
use taffy::TaffyResult;
use taffy::prelude::FromLength;
use taffy::prelude::TaffyMaxContent;

#[derive(Debug, Clone)]
pub struct Itree<T>
where
  T: Inodeable,
{
  // Nodes collection, maps from node ID to its node struct.
  nodes: FoldMap<TreeNodeId, T>,

  // The reference of all common tree node relationships & attributes.
  context: TreeContextRc,
}

impl<T> Itree<T>
where
  T: Inodeable,
{
  pub fn new() -> Self {
    Itree {
      nodes: FoldMap::new(),
      context: TreeContext::to_rc(TreeContext::new()),
    }
  }

  fn _internal_check(&self) {
    debug_assert_eq!(self.context.borrow().len(), self.nodes.len());
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty()
  }

  pub fn root_id(&self) -> TreeNodeId {
    self.context.borrow().root()
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.context.borrow().parent(id)
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self.context.borrow().children(id).unwrap_or_default()
  }

  pub fn node(&self, id: TreeNodeId) -> Option<&T> {
    self.nodes.get(&id)
  }

  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut T> {
    self.nodes.get_mut(&id)
  }

  pub fn context(&self) -> TreeContextRc {
    self.context.clone()
  }

  /// Iterates all nodes in pre-order that starts from the root.
  pub fn iter(&self) -> ItreeIter<'_, T> {
    ItreeIter::new(self, Some(self.context.borrow().root()))
  }
}

impl<T> Default for Itree<T>
where
  T: Inodeable,
{
  fn default() -> Self {
    Self::new()
  }
}

// Insert/Remove {
impl<T> Itree<T>
where
  T: Inodeable,
{
  /// Create a root node, which is the first node in the tree.
  /// Returns the root node ID.
  pub fn add_root<F>(
    &mut self,
    style: Style,
    name: &'static str,
    constructor: F,
  ) -> TaffyResult<TreeNodeId>
  where
    F: FnOnce(
      /* id */ TreeNodeId,
      /* shape */ IRect,
      /* actual_shape */ U16Rect,
    ) -> T,
  {
    self._internal_check();
    debug_assert!(self.nodes.is_empty());

    let (id, shape, actual_shape) = {
      let mut ctx = self.context.borrow_mut();
      let id = ctx.add_root(style, name)?;
      let shape = ctx.shape(id).copied().unwrap();
      let actual_shape = ctx.actual_shape(id).copied().unwrap();
      (id, shape, actual_shape)
    };

    let node = constructor(id, shape, actual_shape);
    self.nodes.insert(id, node);

    Ok(id)
  }

  /// Create a new child node in the tree, and insert it under a parent node.
  /// Returns the child node ID.
  pub fn add_child<F>(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    zindex: usize,
    policy: TruncatePolicy,
    name: &'static str,
    constructor: F,
  ) -> TaffyResult<TreeNodeId>
  where
    F: FnOnce(
      /* id */ TreeNodeId,
      /* shape */ IRect,
      /* actual_shape */ U16Rect,
    ) -> T,
  {
    self._internal_check();
    debug_assert!(self.nodes.contains_key(&parent_id));

    let (id, shape, actual_shape) = {
      let mut ctx = self.context.borrow_mut();
      let id = ctx.add_child(parent_id, style, zindex, policy, name)?;
      let shape = ctx.shape(id).copied().unwrap();
      let actual_shape = ctx.actual_shape(id).copied().unwrap();
      (id, shape, actual_shape)
    };

    let node = constructor(id, shape, actual_shape);
    self.nodes.insert(id, node);

    Ok(id)
  }

  /// Same with [`add_child`](Itree::add_child) method, with default values for
  /// below parameters:
  ///
  /// - zindex: 0
  /// - policy: BRUTAL
  ///
  /// NOTE: For cursor widget node, you should always use the bound policy to
  /// ensure it is inside its parent and avoid been cut off.
  pub fn add_child_with_defaults<F>(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    name: &'static str,
    constructor: F,
  ) -> TaffyResult<TreeNodeId>
  where
    F: FnOnce(
      /* id */ TreeNodeId,
      /* shape */ IRect,
      /* actual_shape */ U16Rect,
    ) -> T,
  {
    self.add_child(
      parent_id,
      style,
      DEFAULT_ZINDEX,
      DEFAULT_TRUNCATE_POLICY,
      name,
      constructor,
    )
  }

  /// Remove a child node.
  /// Returns the removed node.
  ///
  /// NOTE: Never remove the root node.
  pub fn remove_child(&mut self, id: TreeNodeId) -> TaffyResult<Option<T>> {
    self._internal_check();
    debug_assert_ne!(id, self.context.borrow().root());

    match self.nodes.remove(&id) {
      Some(removed_node) => {
        debug_assert!(self.context.borrow().contains(id));
        debug_assert_ne!(self.root_id(), id);
        self.context.borrow_mut().remove_child(id)?;
        Ok(Some(removed_node))
      }
      None => {
        debug_assert!(!self.context.borrow().contains(id));
        Ok(None)
      }
    }
  }
}
// Insert/Remove }

// Movement {

impl<T> Itree<T>
where
  T: Inodeable,
{
  // Raw relative motion.
  fn move_by(
    context: &TreeContext,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> IRect {
    let shape = context.shape(id).unwrap();
    let pos: IPos = shape.min().into();
    Self::move_to(context, id, pos.x() + x, pos.y() + y)
  }

  // Raw absolute motion.
  fn move_to(
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
  /// Returns the new shape after movement if successfully, otherwise
  /// returns `None` if the node doesn't exist or doesn't have a parent.
  ///
  /// NOTE: This motion uses the `RESERVED` policy just like
  /// [TruncatePolicy](TruncatePolicy). If it hits the boundary of its parent
  /// widget, it will simply stop moving to avoid its size been truncated by
  /// its parent.
  pub fn reserved_move_by(
    &self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    let ctx = self.context.borrow();
    let parent_id = ctx.parent(id)?;
    let shape = ctx.shape(id)?;
    let pos: IPos = shape.min().into();
    let new_pos = point!(pos.x() + x, pos.y() + y);
    let new_shape = rect!(
      new_pos.x(),
      new_pos.y(),
      new_pos.x() + shape.width(),
      new_pos.y() + shape.height()
    );
    let parent_actual_shape = ctx.actual_shape(parent_id)?;
    let final_shape =
      shapes::bound_shape(&new_shape, &parent_actual_shape.size());
    let final_pos: IPos = final_shape.min().into();
    let final_x = final_pos.x() - pos.x();
    let final_y = final_pos.y() - pos.y();
    Some(Self::move_by(&ctx, id, final_x, final_y))
  }

  /// Similar to `reserved_move_by`, but moves with absolute position instead
  /// of relative one.
  pub fn reserved_move_to(
    &self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    let ctx = self.context.borrow();
    let parent_id = ctx.parent(id)?;
    let shape = ctx.shape(id).unwrap();
    let new_pos: IPos = point!(x, y);
    let new_shape = rect!(
      new_pos.x(),
      new_pos.y(),
      new_pos.x() + shape.width(),
      new_pos.y() + shape.height()
    );

    let parent_actual_shape = ctx.actual_shape(parent_id)?;
    let final_shape =
      shapes::bound_shape(&new_shape, &parent_actual_shape.size());
    let final_pos: IPos = final_shape.min().into();

    Some(Self::move_to(&ctx, id, final_pos.x(), final_pos.y()))
  }
}
// Movement }

#[derive(Debug)]
/// Iterate all the tree nodes in pre-order.
///
/// For each node, it first visits the node itself, then visits all its
/// children. This also follows the order when rendering the widget tree to
/// terminal device.
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
        let ctx = self.tree.context.borrow();
        self
          .tree
          .children_ids(id)
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
  pub fn new(tree: &'a Itree<T>, start_node_id: Option<TreeNodeId>) -> Self {
    let mut que = VecDeque::new();
    if let Some(id) = start_node_id {
      que.push_back(id);
    }
    Self { tree, que }
  }
}
