//! Internal tree structure that implements the widget tree.

use crate::prelude::*;
use crate::ui::tree::TreeNodeId;
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::internal::arena::*;
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
  arena: TreeArenaRc,
}

impl<T> Itree<T>
where
  T: Inodeable,
{
  pub fn new() -> Self {
    Itree {
      nodes: FoldMap::new(),
      arena: Rc::new(RefCell::new(TreeArena::new())),
    }
  }

  fn _internal_check(&self) {
    debug_assert_eq!(self.arena.borrow().len(), self.nodes.len());
  }

  pub fn len(&self) -> usize {
    self.nodes.len()
  }

  pub fn is_empty(&self) -> bool {
    self.nodes.is_empty()
  }

  pub fn root_id(&self) -> TreeNodeId {
    self.arena.borrow().root()
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.arena.borrow().parent(id)
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self.arena.borrow().children(id).unwrap_or_default()
  }

  pub fn node(&self, id: TreeNodeId) -> Option<&T> {
    self.nodes.get(&id)
  }

  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut T> {
    self.nodes.get_mut(&id)
  }

  /// Get the iterator.
  ///
  /// By default, it iterates in pre-order iterator which starts from the root.
  pub fn iter(&self) -> ItreeIter<'_, T> {
    ItreeIter::new(self, Some(self.arena.borrow().root()))
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
    actual_shape: U16Rect,
    style: Style,
    constructor: F,
    name: &'static str,
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
      let mut arena = self.arena.borrow_mut();
      let id = arena.add_root(actual_shape, style, name)?;
      let attr = arena.attribute(id).unwrap();
      (id, attr.shape, attr.actual_shape)
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
    enabled: bool,
    policy: TruncatePolicy,
    constructor: F,
    name: &'static str,
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
      let mut arena = self.arena.borrow_mut();
      let id =
        arena.add_child(parent_id, style, zindex, enabled, policy, name)?;
      let attr = arena.attribute(id).unwrap();
      (id, attr.shape, attr.actual_shape)
    };

    let node = constructor(id, shape, actual_shape);
    self.nodes.insert(id, node);

    Ok(id)
  }

  /// Same with [`add_child`](Itree::add_child) method, with default values for
  /// below parameters:
  /// - zindex: 0
  /// - enabled: true
  /// - policy: Truncate
  ///
  /// NOTE: For cursor widget node, you should always use the bound policy to
  /// ensure it is inside its parent and avoid been cut off.
  pub fn add_child_with_defaults<F>(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    constructor: F,
    name: &'static str,
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
      DEFAULT_ENABLED,
      TruncatePolicy::BRUTAL,
      constructor,
      name,
    )
  }

  /// Remove a child node.
  /// Returns the removed node.
  ///
  /// NOTE: Never remove the root node.
  pub fn remove_child(&mut self, id: TreeNodeId) -> Option<T> {
    self._internal_check();
    debug_assert_ne!(id, self.arena.borrow().root());

    match self.nodes.remove(&id) {
      Some(removed_node) => {
        debug_assert!(self.relation.contains(id));
        debug_assert!(self.parent_id(id).is_some());
        let parent_id = self.parent_id(id).unwrap();
        self.relation.remove_child(parent_id, id);

        Some(removed_node)
      }
      None => {
        debug_assert!(!self.relation.contains(id));
        None
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
  /// Move node by distance `(x, y)`, the `x`/`y` is the motion distances.
  ///
  /// * The node moves left when `x < 0`.
  /// * The node moves right when `x > 0`.
  /// * The node moves up when `y < 0`.
  /// * The node moves down when `y > 0`.
  ///
  /// NOTE:
  /// 1. The position is relatively based on the node parent.
  /// 2. This operation also updates the shape/position of all descendant nodes, similar to
  ///    [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn move_by(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    match self.nodes.get_mut(&id) {
      Some(node) => {
        let current_shape = *node.shape();
        let current_top_left_pos: IPos = current_shape.min().into();
        self.move_to(
          id,
          current_top_left_pos.x() + x,
          current_top_left_pos.y() + y,
        )
      }
      None => None,
    }
  }

  /// Bounded move node by distance `(x, y)`, the `x`/`y` is the motion distances.
  ///
  /// It works similar to [`move_by`](Itree::move_by), but when a node hits the actual boundary of
  /// its parent, it simply stops moving.
  ///
  /// NOTE:
  /// 1. The position is relatively based on the node parent.
  /// 2. This operation also updates the shape/position of all descendant nodes, similar to
  ///    [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn bounded_move_by(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    match self.parent_id(id) {
      Some(parent_id) => {
        let maybe_parent_actual_size: Option<U16Size> = self
          .nodes
          .get(&parent_id)
          .map(|parent_node| parent_node.actual_shape().size());

        match maybe_parent_actual_size {
          Some(parent_actual_size) => {
            match self.nodes.get_mut(&id) {
              Some(node) => {
                let current_shape = *node.shape();
                let current_top_left_pos: IPos = current_shape.min().into();
                let expected_top_left_pos: IPos = point!(
                  current_top_left_pos.x() + x,
                  current_top_left_pos.y() + y
                );
                let expected_shape = rect!(
                  expected_top_left_pos.x(),
                  expected_top_left_pos.y(),
                  expected_top_left_pos.x() + current_shape.width(),
                  expected_top_left_pos.y() + current_shape.height()
                );

                let final_shape =
                  bound_shape(&expected_shape, &parent_actual_size);
                let final_top_left_pos: IPos = final_shape.min().into();

                // Real movement
                let final_x = final_top_left_pos.x() - current_top_left_pos.x();
                let final_y = final_top_left_pos.y() - current_top_left_pos.y();
                self.move_by(id, final_x, final_y)
              }
              None => None,
            }
          }
          None => None,
        }
      }
      None => None,
    }
  }

  /// Move node to position `(x, y)`, the `(x, y)` is the new position.
  ///
  /// NOTE:
  /// 1. The position is relatively based on the node parent. The `(x, y)` is based on the left-top
  ///    anchor, i.e. the left-top anchor position is `(0, 0)`.
  /// 2. This operation also updates the shape/position of all descendant nodes, similar to
  ///    [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn move_to(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    match self.nodes.get_mut(&id) {
      Some(node) => {
        let current_shape = *node.shape();
        let next_top_left_pos: IPos = point!(x, y);
        let next_shape = rect!(
          next_top_left_pos.x(),
          next_top_left_pos.y(),
          next_top_left_pos.x() + current_shape.width(),
          next_top_left_pos.y() + current_shape.height()
        );
        node.set_shape(&next_shape);

        // Update all the descendants attributes under the `id` node.
        self._update_node_shapes_impl(id, self.parent_id(id).unwrap());

        Some(next_shape)
      }
      None => None,
    }
  }

  /// Bounded move node to position `(x, y)`, the `(x, y)` is the new position.
  ///
  /// It works similar to [`move_by`](Itree::move_by), but when a node hits the actual boundary of
  /// its parent, it simply stops moving.
  ///
  /// NOTE:
  /// 1. The position is relatively based on the node parent. The `(x, y)` is based on the left-top
  ///    anchor, i.e. the left-top anchor position is `(0, 0)`.
  /// 2. This operation also updates the shape/position of all descendant nodes, similar to
  ///    [`insert`](Itree::insert) method.
  ///
  /// # Returns
  ///
  /// 1. The new shape after movement if successfully.
  /// 2. `None` if the node `id` doesn't exist.
  pub fn bounded_move_to(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    match self.parent_id(id) {
      Some(parent_id) => {
        let maybe_parent_actual_size: Option<U16Size> = self
          .nodes
          .get(&parent_id)
          .map(|parent_node| parent_node.actual_shape().size());

        match maybe_parent_actual_size {
          Some(parent_actual_size) => match self.nodes.get_mut(&id) {
            Some(node) => {
              let current_shape = *node.shape();
              let expected_top_left_pos: IPos = point!(x, y);
              let expected_shape = rect!(
                expected_top_left_pos.x(),
                expected_top_left_pos.y(),
                expected_top_left_pos.x() + current_shape.width(),
                expected_top_left_pos.y() + current_shape.height()
              );

              let final_shape =
                bound_shape(&expected_shape, &parent_actual_size);
              let final_top_left_pos: IPos = final_shape.min().into();

              self.move_to(id, final_top_left_pos.x(), final_top_left_pos.y())
            }
            None => None,
          },
          None => None,
        }
      }
      None => None,
    }
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
      for child_id in self.tree.children_ids(id) {
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
