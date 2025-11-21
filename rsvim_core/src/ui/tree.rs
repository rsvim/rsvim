//! The widget tree that manages all the widget components.

pub mod internal;

use crate::inode_dispatcher;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::opt::WindowGlobalOptions;
use crate::ui::widget::window::opt::WindowGlobalOptionsBuilder;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use crate::widget_dispatcher;
pub use internal::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;
use taffy::prelude::FromLength;
use taffy::prelude::TaffyMaxContent;

pub type LayoutNodeId = taffy::NodeId;
pub type TreeNodeId = i32;
pub type TaffyTreeRc = Rc<RefCell<TaffyTree>>;
pub type TaffyTreeWk = Weak<RefCell<TaffyTree>>;

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum TreeNode {
  Root(Dummy),
  Window(Window),
  CommandLine(CommandLine),
}

inode_dispatcher!(TreeNode, Root, Window, CommandLine);
widget_dispatcher!(TreeNode, Root, Window, CommandLine);

#[derive(Debug, Clone)]
/// The widget tree.
///
/// The widget tree manages all UI components and rendering on the canvas, each widget is a tree
/// node on the widget tree, everything inside is the node's children. While the terminal itself is
/// the root widget node.
///
/// # Terms
///
/// * Parent: The parent node.
/// * Child: The child node.
/// * Ancestor: Either the parent, or the parent of some ancestor of the node.
/// * Descendant: Either the child, or the child of some descendant of the node.
/// * Sibling: Other children nodes under the same parent.
///
/// # Guarantees
///
/// ## Ownership
///
/// Parent owns all its children.
///
/// * Children will be destroyed when their parent is.
/// * Coordinate system are relative to their parent's top-left corner, while the absolute
///   coordinates are based on the terminal's top-left corner.
/// * Children are displayed inside their parent's geometric shape, clipped by boundaries. While
///   the size of each node can be logically infinite on the imaginary canvas.
/// * The `visible` and `enabled` attributes of a child are implicitly inherited from it's
///   parent, unless they're explicitly been set.
///
/// ## Priority
///
/// Children have higher priority than their parent to both display and process input events.
///
/// * Children are always displayed on top of their parent, and has higher priority to process
///   a user's input event when the event occurs within the shape of the child. The event will
///   fallback to their parent if the child doesn't process it.
/// * For children that shade each other, the one with higher z-index has higher priority to
///   display and process the input events.
///
/// # Attributes
///
/// ## Shape (position and size)
///
/// A shape can be relative/logical or absolute/actual, and always rectangle. The position is by
/// default relative to its parent top-left corner, and the size is by default logically
/// infinite. While rendering to the terminal device, we need to calculate its absolute position
/// and actual size.
///
/// There're two kinds of positions:
/// * Relative: Based on it's parent's position.
/// * Absolute: Based on the terminal device.
///
/// There're two kinds of sizes:
/// * Logical: An infinite size on the imaginary canvas.
/// * Actual: An actual size bounded by it's parent's actual shape, if it doesn't have a parent,
///   bounded by the terminal device's actual shape.
///
/// The shape boundary uses top-left open, bottom-right closed interval. For example the
/// terminal shape is `((0,0), (10,10))`, the top-left position `(0,0)` is inclusive, i.e.
/// inside the shape, the bottom-right position `(10,10)` is exclusive, i.e. outside the shape.
/// The width and height of the shape is both `10`.
///
/// The absolute/actual shape is calculated with a "copy-on-write" policy. Based on the fact
/// that a widget's shape is often read and rarely modified, thus the "copy-on-write" policy to
/// avoid too many duplicated calculations. i.e. we always calculates a widget's absolute
/// position and actual size right after it's shape is been changed, and also caches the result.
/// Thus we simply get the cached results when need.
///
/// ## Z-index
///
/// The z-index arranges the display priority of the content stack when multiple children
/// overlap on each other, a widget with higher z-index has higher priority to be displayed. For
/// those widgets have the same z-index, the later inserted one will cover the previous inserted
/// ones.
///
/// The z-index only works for the children under the same parent. For a child widget, it always
/// covers/overrides its parent display.
/// To change the visibility priority between children and parent, you need to change the
/// relationship between them.
///
/// For example, now we have two children under the same parent: A and B. A has 100 z-index, B
/// has 10 z-index. Now B has a child: C, with z-index 1000. Even the z-index 1000 > 100 > 10, A
/// still covers C, because it's a sibling of B.
///
/// ## Visible and enabled
///
/// A widget can be visible or invisible. When it's visible, it handles user's input events,
/// processes them and updates the UI contents. When it's invisible, it's just like not existed,
/// so it doesn't handle or process any input events, the UI hides.
///
/// A widget can be enabled or disabled. When it's enabled, it handles input events, processes
/// them and updates the UI contents. When it's disabled, it's just like been fronzen, so it
/// doesn't handle or process any input events, the UI keeps still and never changes.
///
pub struct Tree {
  // Internal relationship.
  relationship: IrelationshipRc,

  // Internal tree.
  base: Itree<TreeNode>,

  // CommandLine node ID.
  command_line_id: Option<TreeNodeId>,

  // All window node IDs.
  window_ids: BTreeSet<TreeNodeId>,

  // The *current* window node ID.
  //
  // The **current** window means it contains cursor, even when user is typing
  // commands in cmdline widget, the cursor is actually in the cmdline widget,
  // the **current** window is the latest window that contains the cursor.
  current_window_id: Option<TreeNodeId>,

  // Global window options.
  global_options: WindowGlobalOptions,
  global_local_options: WindowOptions,
}

arc_mutex_ptr!(Tree);

// pub type TreeIter<'a> = ItreeIter<'a, TreeNode>;
// pub type TreeIterMut<'a> = ItreeIterMut<'a, TreeNode>;

// Node {
impl Tree {
  /// Make a widget tree.
  ///
  /// NOTE: The root node is created along with the tree.
  pub fn new(canvas_size: U16Size) -> TaffyResult<Self> {
    let relationship = Irelationship::to_rc(Irelationship::new());

    let (root_loid, root_shape) = {
      let root_style = Style {
        size: taffy::Size {
          width: taffy::Dimension::from_length(canvas_size.width()),
          height: taffy::Dimension::from_length(canvas_size.height()),
        },
        flex_direction: taffy::FlexDirection::Column,
        ..Default::default()
      };

      let mut lo = lotree.borrow_mut();
      let root_loid = lo.new_leaf(root_style)?;
      lo.compute_layout(root_loid, taffy::Size::MAX_CONTENT)?;
      let root_layout = lo.layout(root_loid)?;
      let root_shape = rect_from_layout!(root_layout, u16);
      (root_loid, root_shape)
    };

    let root = Dummy::new(root_loid, root_shape);
    let root_node = TreeNode::Root(root);
    Ok(Tree {
      relationship,
      base: Itree::new(lotree, root_node),
      command_line_id: None,
      window_ids: BTreeSet::new(),
      current_window_id: None,
      global_options: WindowGlobalOptionsBuilder::default().build().unwrap(),
      global_local_options: WindowOptionsBuilder::default().build().unwrap(),
    })
  }

  pub fn lotree(&self) -> TaffyTreeRc {
    self.base.lotree()
  }

  /// Root node ID.
  pub fn root_id(&self) -> TreeNodeId {
    self.base.root_id()
  }

  /// Root node ID.
  pub fn root_loid(&self) -> LayoutNodeId {
    self.base.root_loid()
  }

  /// Get the parent ID by a node `id`.
  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.base.parent_id(id)
  }

  /// Get the children IDs by a node `id`.
  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self.base.children_ids(id)
  }

  /// Get the node struct by its `id`.
  pub fn node(&self, id: TreeNodeId) -> Option<&TreeNode> {
    self.base.node(id)
  }

  /// Get mutable node struct by its `id`.
  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut TreeNode> {
    self.base.node_mut(id)
  }

  // /// See [`Itree::iter`].
  // pub fn iter(&self) -> TreeIter {
  //   self.base.iter()
  // }
  //
  // /// See [`Itree::iter_mut`].
  // pub fn iter_mut(&mut self) -> TreeIterMut {
  //   self.base.iter_mut()
  // }

  /// Get command-line node ID.
  pub fn command_line_id(&self) -> Option<TreeNodeId> {
    self.command_line_id
  }

  /// Get current window node ID.
  /// NOTE: A window is called the current window because it has cursor inside it. But when user is
  /// in command-line mode, the cursor widget is actually inside the command-line widget, not in
  /// window. Mean while the **current** window is actually the **previous current** window.
  pub fn current_window_id(&self) -> Option<TreeNodeId> {
    self.current_window_id
  }

  /// Set current window node ID.
  ///
  /// NOTE: When the node ID is not `None`, it must be a valid tree node, existing in current tree,
  /// and it must be a window widget.
  pub fn set_current_window_id(
    &mut self,
    window_id: Option<TreeNodeId>,
  ) -> Option<TreeNodeId> {
    if cfg!(debug_assertions) {
      match window_id {
        Some(window_id) => {
          debug_assert!(self.node_mut(window_id).is_some());
          debug_assert!(matches!(
            self.node_mut(window_id).unwrap(),
            TreeNode::Window(_)
          ));
        }
        None => { /* */ }
      }
    }
    let old = self.current_window_id;
    self.current_window_id = window_id;
    old
  }

  /// Get all the window widget IDs.
  pub fn window_ids(&self) -> &BTreeSet<TreeNodeId> {
    &self.window_ids
  }
}
// Node {

// Widget {
impl Tree {
  /// Window widget.
  pub fn window(&self, window_id: TreeNodeId) -> Option<&Window> {
    match self.node(window_id) {
      Some(window_node) => {
        debug_assert!(matches!(window_node, TreeNode::Window(_)));
        match window_node {
          TreeNode::Window(w) => {
            debug_assert_eq!(w.id(), window_id);
            Some(w)
          }
          _ => unreachable!(), // Other variants not allowed.
        }
      }
      None => None,
    }
  }

  /// Mutable window widget.
  pub fn window_mut(&mut self, window_id: TreeNodeId) -> Option<&mut Window> {
    match self.node_mut(window_id) {
      Some(window_node) => {
        debug_assert!(matches!(window_node, TreeNode::Window(_)));
        match window_node {
          TreeNode::Window(w) => {
            debug_assert_eq!(w.id(), window_id);
            Some(w)
          }
          _ => unreachable!(), // Other variants not allowed.
        }
      }
      None => None,
    }
  }

  // Current window widget.
  pub fn current_window(&self) -> Option<&Window> {
    match self.current_window_id {
      Some(current_window_id) => self.window(current_window_id),
      None => None,
    }
  }

  // Mutable current window widget.
  pub fn current_window_mut(&mut self) -> Option<&mut Window> {
    match self.current_window_id {
      Some(current_window_id) => self.window_mut(current_window_id),
      None => None,
    }
  }

  // Command-line widget.
  pub fn command_line(&self) -> Option<&CommandLine> {
    match self.command_line_id {
      Some(cmdline_id) => {
        debug_assert!(self.node(cmdline_id).is_some());
        let cmdline_node = self.node(cmdline_id).unwrap();
        debug_assert!(matches!(cmdline_node, TreeNode::CommandLine(_)));
        match cmdline_node {
          TreeNode::CommandLine(w) => {
            debug_assert_eq!(w.id(), cmdline_id);
            Some(w)
          }
          _ => unreachable!(),
        }
      }
      None => None,
    }
  }

  // Mutable command-line widget.
  pub fn command_line_mut(&mut self) -> Option<&mut CommandLine> {
    match self.command_line_id {
      Some(cmdline_id) => {
        debug_assert!(self.node_mut(cmdline_id).is_some());
        let cmdline_node = self.node_mut(cmdline_id).unwrap();
        debug_assert!(matches!(cmdline_node, TreeNode::CommandLine(_)));
        match cmdline_node {
          TreeNode::CommandLine(w) => {
            debug_assert_eq!(w.id(), cmdline_id);
            Some(w)
          }
          _ => unreachable!(),
        }
      }
      None => None,
    }
  }
}
// Widget }

// Insert/Remove {
impl Tree {
  fn insert_guard(&mut self, node: &TreeNode) {
    match node {
      TreeNode::CommandLine(command_line) => {
        // When insert command-line widget, update `command_line_id`.
        self.command_line_id = Some(command_line.id());
      }
      TreeNode::Window(window) => {
        // When insert window widget, update `window_ids`.
        self.window_ids.insert(window.id());
      }
      _ => { /* Skip */ }
    }
  }

  /// See [`Itree::insert`].
  pub fn insert(
    &mut self,
    parent_id: TreeNodeId,
    child_node: TreeNode,
  ) -> Option<TreeNode> {
    self.insert_guard(&child_node);
    self.base.insert(parent_id, child_node)
  }

  fn remove_guard(&mut self, id: TreeNodeId) {
    if self.command_line_id == Some(id) {
      self.command_line_id = None;
    }
    self.window_ids.remove(&id);
    if self.current_window_id == Some(id)
      && let Some(last_window_id) = self.window_ids.last()
    {
      self.current_window_id = Some(*last_window_id);
    }
  }

  /// See [`Itree::remove`].
  pub fn remove(&mut self, id: TreeNodeId) -> Option<TreeNode> {
    self.remove_guard(id);
    self.base.remove(id)
  }
}
// Insert/Remove }

// Movement {
impl Tree {
  /// Bounded move by x(columns) and y(rows). This is simply a wrapper method on
  /// [`Itree::bounded_move_by`].
  pub fn bounded_move_by(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    self.base.bounded_move_by(id, x, y)
  }

  /// Bounded move to position x(columns) and y(rows). This is simply a wrapper method on
  /// [`Itree::bounded_move_to`].
  pub fn bounded_move_to(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    self.base.bounded_move_to(id, x, y)
  }
}
// Movement }

// Global options {
impl Tree {
  pub fn global_options(&self) -> &WindowGlobalOptions {
    &self.global_options
  }

  pub fn global_options_mut(&mut self) -> &mut WindowGlobalOptions {
    &mut self.global_options
  }

  pub fn set_global_options(&mut self, options: &WindowGlobalOptions) {
    self.global_options = *options;
  }

  pub fn global_local_options(&self) -> &WindowOptions {
    &self.global_local_options
  }

  pub fn global_local_options_mut(&mut self) -> &mut WindowOptions {
    &mut self.global_local_options
  }

  pub fn set_global_local_options(&mut self, options: &WindowOptions) {
    self.global_local_options = *options;
  }
}
// Global options }

// Draw {
impl Tree {
  /// Draw the widget tree to canvas.
  pub fn draw(&self, canvas: CanvasArc) {
    let mut canvas = lock!(canvas);
    for node in self.base.iter() {
      // trace!("Draw tree:{:?}", node);
      if !node.visible() {
        continue;
      }
      node.draw(&mut canvas);
    }
  }
}
// Draw }
