//! The widget tree that manages all the widget components.

pub mod internal;

use crate::inode_dispatcher;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::panel::Panel;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::opt::WindowGlobalOptions;
use crate::ui::widget::window::opt::WindowGlobalOptionsBuilder;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use crate::widget_dispatcher;
pub use internal::*;

pub type TreeNodeId = i32;

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum TreeNode {
  Root(Panel),
  Window(Window),
  CommandLine(CommandLine),
}

inode_dispatcher!(TreeNode, Root, Window, CommandLine);
widget_dispatcher!(TreeNode, Root, Window, CommandLine);

#[derive(Debug, Clone)]
/// The UI/widget tree.
///
/// This tree manages all UI components and renders them on the canvas, each
/// widget is a node on the tree, everything inside is the node's children.
/// While the terminal itself is the root widget node.
///
/// > An element in UI tree can be called node, widget, component or whatever.
///
/// The tree guarantees several constraints on all nodes:
///
/// ## Ownership
///
/// A parent node owns its children, more specifically:
///
/// * Children will be destroyed when their parent is.
/// * There are two location coordinates: relative location based on parent's
///   location (top-left corner), absolute location based on terminal.
/// * Children must be displayed inside their parent's geometric shape,
///   truncated by their parent boundaries.
/// * Z-index/enabled attributes will affected all its descendant nodes.
///
/// ## Rendering Order
///
/// - Children have higher priority to render on terminal than parent.
/// - For all the children under the same parent, nodes with higher Z-index
///   have higher priority than the ones with lower Z-index.
/// - Disabled nodes (i.e. with `style { display: none}`) are not rendered.
///
/// ## Attributes
///
/// ### Shape/Position/Size
///
/// A shape is always a rectangle, it can be relative based on its parent or
/// absolute (actual) based on terminal. We use relative shape for an easier
/// code logic, use absolute shape when rendering it to terminal.
///
/// ### Visible/Enabled
///
/// A widget can be visible or invisible, enabled or disabled.
pub struct Tree {
  // Internal implementation.
  base: Itree<TreeNode>,

  // [`CommandLine`](crate::ui::widget::command_line::CommandLine) node ID.
  command_line_id: Option<TreeNodeId>,

  // All [`Window`](crate::ui::widget::Window) node IDs.
  window_ids: BTreeSet<TreeNodeId>,

  // The *current* window node ID.
  //
  // The **current** window means user is focused on the window widget, i.e. it contains the
  // cursor, since the cursor is like the mouse on the screen.
  //
  // But when user inputs commands in cmdline widget, the cursor widget will move to the cmdline
  // widget. But we still keeps the **current window**, this field is actually the **previous**
  // current window.
  current_window_id: Option<TreeNodeId>,

  // Global options for windows.
  global_options: WindowGlobalOptions,

  // Global-local options for windows.
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
  pub fn new(canvas_size: U16Size) -> Self {
    let shape = rect_from_size!(canvas_size);
    let shape = rect_as!(shape, isize);

    let mut base = Itree::new();

    let root = Panel::new(shape);
    let root_node = TreeNode::Root(root);
    base.add_root(root_node);

    Tree {
      base,
      command_line_id: None,
      window_ids: BTreeSet::new(),
      current_window_id: None,
      global_options: WindowGlobalOptionsBuilder::default().build().unwrap(),
      global_local_options: WindowOptionsBuilder::default().build().unwrap(),
    }
  }

  /// Nodes count, include the root node.
  pub fn len(&self) -> usize {
    self.base.len()
  }

  /// Whether the tree is empty.
  pub fn is_empty(&self) -> bool {
    self.base.is_empty()
  }

  /// Root node ID.
  pub fn root_id(&self) -> TreeNodeId {
    self.base.root_id()
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
      _ => {}
    }
  }

  /// See [`Itree::insert`].
  pub fn insert(
    &mut self,
    parent_id: TreeNodeId,
    child_node: TreeNode,
  ) -> Option<TreeNode> {
    self.insert_guard(&child_node);
    self.base.add_child(parent_id, child_node)
  }

  /// See [`Itree::bounded_insert`].
  pub fn bounded_insert(
    &mut self,
    parent_id: TreeNodeId,
    child_node: TreeNode,
  ) -> Option<TreeNode> {
    self.insert_guard(&child_node);
    self.base.bounded_insert(parent_id, child_node)
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
    self.base.remove_child(id)
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
    self.base.reserved_move_to(id, x, y)
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
