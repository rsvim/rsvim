//! The widget tree that manages all the widget components.

pub mod internal;

use crate::inode_enum_dispatcher;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::root::RootContainer;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::opt::WindowGlobalOptions;
use crate::ui::widget::window::opt::WindowGlobalOptionsBuilder;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use crate::widget_enum_dispatcher;
pub use internal::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::rc::Weak;
use taffy::Style;
use taffy::TaffyResult;
use taffy::TaffyTree;

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum TreeNode {
  RootContainer(RootContainer),
  Window(Window),
  CommandLine(CommandLine),
}

inode_enum_dispatcher!(TreeNode, RootContainer, Window, CommandLine);
widget_enum_dispatcher!(TreeNode, RootContainer, Window, CommandLine);

pub type TaffyTreeRc = Rc<RefCell<TaffyTree>>;
pub type TaffyTreeWk = Weak<RefCell<TaffyTree>>;

pub fn new_layout_tree() -> TaffyTreeRc {
  let mut layout_tree = TaffyTree::new();
  layout_tree.disable_rounding();
  Rc::new(RefCell::new(layout_tree))
}

#[derive(Debug, Clone)]
/// The widget tree (UI tree).
///
/// The widget tree manages all UI widgets and rendering on the canvas, each
/// widgiet is a node on the tree, the tree has a root node, and all other
/// nodes inside is the root node's descendants. The root node is the terminal
/// itself, while each node inside renders a part of the terminal.
///
/// We use [taffy] to manage the parent-child relationships among all the
/// nodes, and calculate layout for the whole TUI. The tree structure contains
/// a [TaffyTree] pointer. Each node holds a weak reference point to that
/// [TaffyTree], and also a [taffy::Style] to indicate what style this node
/// wants to be, a [taffy::Layout] to cache the layout result that how this
/// node is going to render itself.
///
/// # Terms
///
/// * Parent: The parent node.
/// * Child: The child node.
/// * Ancestor: Either the parent, or the parent of some ancestor of the node.
/// * Descendant: Either the child, or the child of some descendant of the node.
/// * Sibling: Other children nodes under the same parent.
///
/// Taffy implements several layout algorithms in
/// [CSS](https://developer.mozilla.org/en-US/docs/Web/CSS) specification:
///
/// - flexbox
/// - grid
/// - block
///
/// They are just right to laying out Rsvim UI widgets as well. But layout just
/// tells a node where it should be rendering, it is still need to implement
/// the rendering method by itself.
///
/// # Ownership
///
/// Parent owns its children:
///
/// * Children will be destroyed when their parent is.
/// * Children are displayed inside their parent's geometric shape, clipped by
///   boundaries. While the size of each node can be logically infinite on the
///   imaginary canvas.
/// * The `visible` and `enabled` attributes of a child are implicitly
///   inherited from it's parent, unless they're explicitly been set.
///
/// # Priority
///
/// Children have higher priority than their parent to both display and process
/// input events:
///
/// * Children are always displayed on top of their parent, and has higher
///   priority to process a user's input event when the event occurs within the
///   shape of the child. The event will fallback to their parent if the child
///   doesn't process it.
/// * For children that shade each other, the one with higher z-index has
///   higher priority to display and process the input events.
///
/// ## Visible/Enabled
///
/// A widget can be hidden/disabled, this is useful for some special use cases.
/// For example, when implementing the "command-line" UI widget, we actually
/// have multiple command-line widgets:
/// - The "input" widget for receiving user's input command contents.
/// - The "message" widget for printing Rsvim echoing messages.
/// - The "search" widget for searching forward/backward.
///
/// At a certain time, only 1 of these 3 widgets is visible/enabled, the other
/// 2 are hidden/disabled.
/// Thus we have to remove the other 2 nodes from the layout tree, the make
/// sure they won't break our TUI layout.
///
pub struct Tree {
  // Widget nodes.
  nodes: FoldMap<TreeNodeId, TreeNode>,

  // Maps widget node ID => layout node ID.
  tree_node_ids: FoldMap<TreeNodeId, LayoutNodeId>,

  // Maps layout node ID => widget node ID.
  layout_node_ids: FoldMap<LayoutNodeId, TreeNodeId>,

  // Root node ID.
  root_id: TreeNodeId,

  // Root layout node ID.
  root_layout_id: LayoutNodeId,

  // Canvas size.
  size: U16Size,

  // Layout tree.
  layout_tree: TaffyTreeRc,

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
  pub fn new(canvas_size: U16Size) -> TaffyResult<Self> {
    let mut layout = TaffyTree::new();
    layout.disable_rounding();
    let root_style = Style {
      size: taffy::Size {
        width: taffy::Dimension::length(canvas_size.width() as f32),
        height: taffy::Dimension::length(canvas_size.height() as f32),
      },
      ..Default::default()
    };
    match layout.new_leaf(root_style) {
      Ok(root_layout_id) => Ok(Tree {
        nodes: FoldMap::new(),
        root_id: next_node_id(),
        root_layout_id,
        size: canvas_size,
        layout_tree: Rc::new(RefCell::new(layout)),
        command_line_id: None,
        window_ids: BTreeSet::new(),
        current_window_id: None,
        global_options: WindowGlobalOptionsBuilder::default().build().unwrap(),
        global_local_options: WindowOptionsBuilder::default().build().unwrap(),
      }),
      Err(e) => Err(e),
    }
  }

  /// Root node ID.
  pub fn root_id(&self) -> TreeNodeId {
    self.root_id
  }

  /// Root layout node ID.
  pub fn root_layout_id(&self) -> LayoutNodeId {
    self.tree_node_ids.get(&self.root_id).unwrap()
  }

  /// Get node by its `id`.
  pub fn node(&self, id: TreeNodeId) -> Option<&TreeNode> {
    self.nodes.get(&id)
  }

  /// Get mutable node by its `id`.
  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut TreeNode> {
    self.nodes.get_mut(&id)
  }

  pub fn layout_id(&self, id: TreeNodeId) -> Option<&LayoutNodeId> {
    self.tree_node_ids.get(&id)
  }

  pub fn node_id(&self, layout_id: LayoutNodeId) -> Option<&TreeNodeId> {
    self.layout_node_ids.get(&layout_id)
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
  ///
  /// NOTE: A window is called the current window because it has cursor inside
  /// it. But when user is in command-line mode, the cursor widget is actually
  /// inside the command-line widget, not in window. Mean while the **current**
  /// window is actually the **last current** window.
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
          if self.tree.layout_node_ids.contains_key(&child_layout_id) {
            self.que.push_back(child_layout_id);
          }
        }
      }
      let node_id = self.tree.layout_node_ids.get(&layout_id).unwrap();
      self.tree.node(*node_id)
    } else {
      None
    }
  }
}

impl<'a, T> TreeIter<'a, T>
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
