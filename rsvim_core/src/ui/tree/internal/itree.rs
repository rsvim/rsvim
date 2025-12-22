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
    self.nodes.len() <= 1
  }

  pub fn root_id(&self) -> TreeNodeId {
    self.arena.borrow().root_id()
  }

  pub fn node_ids(&self) -> Vec<TreeNodeId> {
    self.nodes.keys().copied().collect()
  }

  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.relation.parent(id)
  }

  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self.relation.children(id).unwrap_or_default()
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
    ItreeIter::new(self, Some(self.relation.root_id()))
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
  fn _update_node_shapes_impl(
    &mut self,
    start_id: TreeNodeId,
  ) -> TaffyResult<()> {
    let mut q: VecDeque<TreeNodeId> = VecDeque::new();
    q.push_back(start_id);

    // Iterate all descendants, and update their shape/actual_shape.
    while let Some(id) = q.pop_front() {
      let layout = self.ta.borrow().layout(id)?.clone();
      let policy = self.node(id).unwrap().truncate_policy();
      let shape = rect_from_layout!(layout);
      let shape = self.calculate_shape(id, &shape, policy);
      let actual_shape = self.calculate_actual_shape(id, &shape);
      self.node_mut(id).unwrap().set_shape(shape);
      self.node_mut(id).unwrap().set_actual_shape(actual_shape);

      if let Ok(ta_children_ids) = self.ta.borrow().children(id) {
        for ta_child in ta_children_ids {
          q.push_back(ta_child);
        }
      }
    }

    Ok(())
  }

  fn _update_node_shapes_except(
    &mut self,
    parent_id: TreeNodeId,
    except_id: TreeNodeId,
  ) -> TaffyResult<()> {
    let ta_children_ids = self.ta.borrow().children(parent_id);
    if let Ok(ta_children_ids) = ta_children_ids {
      for ta_child in ta_children_ids {
        // We don't have to update `except_id` again because we had just done
        // it.
        if ta_child != except_id {
          self._update_node_shapes_impl(ta_child)?;
        }
      }
    }
    Ok(())
  }

  fn clamp_shape(shape: &IRect) -> IRect {
    let min_x = num_traits::clamp_min(shape.min().x, 0);
    let min_y = num_traits::clamp_min(shape.min().y, 0);
    let max_x = num_traits::clamp_min(shape.max().x, min_x);
    let max_y = num_traits::clamp_min(shape.max().y, min_y);
    rect!(min_x, min_y, max_x, max_y)
  }

  fn calculate_shape(
    &self,
    id: TreeNodeId,
    shape: &IRect,
    policy: TruncatePolicy,
  ) -> IRect {
    match self.parent_id(id) {
      Some(parent_id) => {
        let parent_actual_shape = self.node(parent_id).unwrap().actual_shape();
        match policy {
          TruncatePolicy::NEGLECT => {
            truncate_shape(&shape, &parent_actual_shape.size())
          }
          TruncatePolicy::PRESERVE => {
            bound_shape(&shape, &parent_actual_shape.size())
          }
        }
      }
      None => Self::clamp_shape(shape),
    }
  }

  pub fn calculate_actual_shape(
    &self,
    id: TreeNodeId,
    shape: &IRect,
  ) -> U16Rect {
    match self.parent_id(id) {
      Some(parent_id) => {
        let parent_actual_shape = self.node(parent_id).unwrap().actual_shape();
        convert_relative_to_absolute(&shape, &parent_actual_shape)
      }
      None => {
        rect_as!(shape, u16)
      }
    }
  }

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

    let (id, shape) = {
      let mut ta = self.ta.borrow_mut();
      let id = ta.new_leaf(style)?;
      ta.compute_layout(
        id,
        taffy::Size {
          width: taffy::AvailableSpace::from_length(
            actual_shape.size().width(),
          ),
          height: taffy::AvailableSpace::from_length(
            actual_shape.size().height(),
          ),
        },
      )?;
      let layout = ta.layout(id)?;
      let shape = rect_from_layout!(layout);
      let shape = Self::clamp_shape(&shape);
      (id, shape)
    };

    self.relation.add_root(id, name);
    self.relation.set_children_zindex(id, DEFAULT_ZINDEX);

    let mut node = constructor(id, shape, actual_shape);
    node.set_zindex(DEFAULT_ZINDEX);
    node.set_enabled(DEFAULT_ENABLED);
    node.set_shape(shape);
    node.set_actual_shape(actual_shape);
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

    let (id, shape) = {
      let mut ta = self.ta.borrow_mut();
      if enabled {
        // Detect whether TaffyTree currently is on the Z-index layer, clear and
        // re-insert all the children nodes that are in the same layer of
        // current `zindex`.
        let children_zindex = self.relation.children_zindex(parent_id);
        if children_zindex.is_none() || children_zindex.unwrap() != zindex {
          // Clear all children nodes under this parent.
          ta.set_children(parent_id, &[]);

          // Re-inesrt all children nodes equals to the `zindex` to this parent.
          for child in self.children_ids(parent_id) {
            debug_assert!(self.node(child).is_some());
            let child_zindex = self.node(child).unwrap().zindex();
            if child_zindex == zindex {
              ta.add_child(parent_id, child);
            }
          }
          self.relation.set_children_zindex(parent_id, zindex);
        }

        let id = ta.new_with_parent(style, parent_id)?;
        ta.compute_layout(self.relation.root_id(), taffy::Size::MAX_CONTENT)?;
        let layout = ta.layout(id)?;
        (id, rect_from_layout!(layout))
      } else {
        // Where the child node is disabled, we simply mock it with parent's
        // shape.
        (ta.new_leaf(style)?, *self.node(parent_id).unwrap().shape())
      }
    };

    self.relation.add_child(parent_id, id, name);

    let shape = self.calculate_shape(id, &shape, policy);
    let actual_shape = self.calculate_actual_shape(id, &shape);
    let mut node = constructor(id, shape, actual_shape);
    node.set_zindex(zindex);
    node.set_enabled(enabled);
    node.set_shape(shape);
    node.set_actual_shape(actual_shape);
    self.nodes.insert(id, node);

    // After this new child node is created, it may also affected the other
    // children nodes under the same parent with the same Z-index, because the
    // layout is been changed.
    // Thus we have to update both shape and actual_shape for all the children
    // nodes under the parent, except this newly created child node because we
    // just had done it.
    self._update_node_shapes_except(parent_id, id)?;

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
      TruncatePolicy::NEGLECT,
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
    debug_assert_ne!(id, self.relation.root_id());

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
