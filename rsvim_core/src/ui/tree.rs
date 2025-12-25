//! The widget tree that manages all the widget components.

pub mod internal;

use std::sync::Arc;

use crate::buf::BufferWk;
use crate::inode_dispatcher;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::viewport::Viewport;
use crate::ui::widget::Widgetable;
use crate::ui::widget::cmdline::Cmdline;
use crate::ui::widget::cmdline::indicator::CmdlineIndicator;
use crate::ui::widget::cmdline::input::CmdlineInput;
use crate::ui::widget::cmdline::message::CmdlineMessage;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::panel::Panel;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::opt::WindowGlobalOptions;
use crate::ui::widget::window::opt::WindowGlobalOptionsBuilder;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use crate::widget_dispatcher;
pub use internal::*;
use taffy::Style;
use taffy::TaffyResult;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;

pub type TreeNodeId = i32;

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum TreeNode {
  Root(Panel),
  Cursor(Cursor),
  Window(Window),
  WindowContent(WindowContent),
  Cmdline(Cmdline),
  CmdlineIndicator(CmdlineIndicator),
  CmdlineInput(CmdlineInput),
  CmdlineMessage(CmdlineMessage),
}

inode_dispatcher!(
  TreeNode,
  Root,
  Cursor,
  Window,
  WindowContent,
  Cmdline,
  CmdlineIndicator,
  CmdlineInput,
  CmdlineMessage
);
widget_dispatcher!(
  TreeNode,
  Root,
  Cursor,
  Window,
  WindowContent,
  Cmdline,
  CmdlineIndicator,
  CmdlineInput,
  CmdlineMessage
);

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
/// * There are two position coordinates: relative position based on parent's
///   position (top-left corner), absolute position based on terminal.
/// * Children must be displayed inside their parent's geometric shape,
///   truncated by their parent boundaries.
/// * If a node is disabled, then all its descendant nodes are disabled.
///
/// ## Rendering Order
///
/// A node with higher rendering priority will be rendered after those with
/// lower rendering priority:
///
/// - Children have higher priority to render on terminal than parent.
/// - For all the children under the same parent, nodes with higher Z-index
///   have higher priority than the ones with lower Z-index.
/// - Disabled nodes (i.e. with `style { display: none}`) are not rendered.
///
/// ## Attributes
///
/// ### Position/size/shape
///
/// A node's shape is always a rectangle, it's position can be either relative
/// based on its parent or absolute based on terminal. Relative position is
/// easier for processing user logic, while absolute position is easier for
/// rendering the UI widget on the terminal.
///
/// ### Z-index/enabled
///
/// By default a node Z-index is 0, and it is enabled. You can raise rendering
/// priority by set a bigger value to its Z-index, or mark it as disabled to
/// not render it.
pub struct Tree {
  // Internal implementation.
  base: Itree<TreeNode>,

  // Cursor node ID.
  cursor_id: Option<TreeNodeId>,

  // Command-line node ID.
  cmdline_id: Option<TreeNodeId>,

  // Window ID collection.
  window_ids: BTreeSet<TreeNodeId>,

  // *Current* window ID.
  //
  // *Current* window means it contains the cursor, e.g. user is focusing on
  // it, because the cursor in vim editor is like a mouse on the screen.
  //
  // But when user starts typing commands in the command-line, cursor actually
  // moves to command-line widget. But we still saves the *current* window, now
  // it is more like a *previous* window.
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
    let mut base = Itree::new();

    let style = Style {
      size: taffy::Size {
        width: taffy::Dimension::from_length(canvas_size.width()),
        height: taffy::Dimension::from_length(canvas_size.height()),
      },
      flex_direction: taffy::FlexDirection::Column,
      ..Default::default()
    };

    base.new_root(style, "Panel", |id, context, _shape, _actual_shape| {
      let root = Panel::new(id, context);
      TreeNode::Root(root)
    })?;

    Ok(Tree {
      base,
      cursor_id: None,
      cmdline_id: None,
      window_ids: BTreeSet::new(),
      current_window_id: None,
      global_options: WindowGlobalOptionsBuilder::default().build().unwrap(),
      global_local_options: WindowOptionsBuilder::default().build().unwrap(),
    })
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

  /// Get cursor ID.
  pub fn cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id
  }

  /// Get command-line ID.
  pub fn cmdline_id(&self) -> Option<TreeNodeId> {
    self.cmdline_id
  }

  /// Get current window ID.
  pub fn current_window_id(&self) -> Option<TreeNodeId> {
    self.current_window_id
  }

  /// Set current window ID.
  /// NOTE: It must be a valid window node.
  pub fn set_current_window_id(
    &mut self,
    id: Option<TreeNodeId>,
  ) -> Option<TreeNodeId> {
    if cfg!(debug_assertions)
      && let Some(id) = id
    {
      debug_assert!(self.node(id).is_some());
      debug_assert!(self.window_ids.contains(&id));
      debug_assert!(matches!(self.node(id).unwrap(), TreeNode::Window(_)));
    }
    let old = self.current_window_id;
    self.current_window_id = id;
    old
  }

  /// Get all window IDs.
  pub fn window_ids(&self) -> &BTreeSet<TreeNodeId> {
    &self.window_ids
  }
}
// Node {

// Widget {
impl Tree {
  /// Cursor widget.
  /// It panics if cursor doesn't exist.
  pub fn cursor(&self) -> &Cursor {
    let cursor_id = self.cursor_id.unwrap();
    let n = self.node(cursor_id).unwrap();
    debug_assert!(matches!(n, TreeNode::Cursor(_)));
    match n {
      TreeNode::Cursor(c) => {
        debug_assert_eq!(c.id(), cursor_id);
        c
      }
      _ => unreachable!(),
    }
  }

  /// Mutable cursor widget.
  /// It panics if cursor doesn't exist.
  pub fn cursor_mut(&mut self) -> &mut Cursor {
    let cursor_id = self.cursor_id.unwrap();
    let n = self.node_mut(cursor_id).unwrap();
    debug_assert!(matches!(n, TreeNode::Cursor(_)));
    match n {
      TreeNode::Cursor(c) => {
        debug_assert_eq!(c.id(), cursor_id);
        c
      }
      _ => unreachable!(),
    }
  }

  /// Window widget.
  /// It panics if window doesn't exist.
  pub fn window(&self, id: TreeNodeId) -> &Window {
    let n = self.node(id).unwrap();
    debug_assert!(matches!(n, TreeNode::Window(_)));
    match n {
      TreeNode::Window(w) => {
        debug_assert_eq!(w.id(), id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Mutable window widget.
  /// It panics if window doesn't exist.
  pub fn window_mut(&mut self, id: TreeNodeId) -> &mut Window {
    let n = self.node_mut(id).unwrap();
    debug_assert!(matches!(n, TreeNode::Window(_)));
    match n {
      TreeNode::Window(w) => {
        debug_assert_eq!(w.id(), id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Current window widget.
  /// It panics if current window doesn't exist.
  pub fn current_window(&self) -> &Window {
    self.window(self.current_window_id.unwrap())
  }

  /// Mutable current window widget.
  /// It panics if current window doesn't exist.
  pub fn current_window_mut(&mut self) -> &mut Window {
    self.window_mut(self.current_window_id.unwrap())
  }

  /// Command-line widget.
  /// It panics if command-line doesn't exist.
  pub fn cmdline(&self) -> &Cmdline {
    let cmdline_id = self.cmdline_id.unwrap();
    let n = self.node(cmdline_id).unwrap();
    debug_assert!(matches!(n, TreeNode::Cmdline(_)));
    match n {
      TreeNode::Cmdline(c) => {
        debug_assert_eq!(c.id(), cmdline_id);
        c
      }
      _ => unreachable!(),
    }
  }

  // Mutable command-line widget.
  /// It panics if command-line doesn't exist.
  pub fn cmdline_mut(&mut self) -> &mut Cmdline {
    let cmdline_id = self.cmdline_id.unwrap();
    let n = self.node_mut(cmdline_id).unwrap();
    debug_assert!(matches!(n, TreeNode::Cmdline(_)));
    match n {
      TreeNode::Cmdline(c) => {
        debug_assert_eq!(c.id(), cmdline_id);
        c
      }
      _ => unreachable!(),
    }
  }
}
// Widget }

// Insert/Remove {
impl Tree {
  fn _insert_guard(&mut self, node: &TreeNode) {
    match node {
      TreeNode::Cursor(c) => {
        self.cursor_id = Some(c.id());
      }
      TreeNode::Cmdline(c) => {
        self.cmdline_id = Some(c.id());
      }
      TreeNode::Window(w) => {
        self.window_ids.insert(w.id());
      }
      _ => {}
    }
  }

  /// See [`Itree::insert`].
  pub fn add_window(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    opts: WindowOptions,
    buffer: BufferWk,
  ) -> TaffyResult<TreeNodeId> {
    // Create a mock viewport to help create window content.
    let mocked_viewport = {
      let mocked_size = size!(1, 1);
      let buffer = buffer.upgrade().unwrap();
      let buffer = lock!(buffer);
      let viewport = Viewport::view(&opts, buffer.text(), &mocked_size, 0, 0);
      Viewport::to_arc(viewport)
    };

    // window content widget
    let content_style = Style {
      size: taffy::Size {
        width: taffy::Dimension::from_percent(1.0),
        height: taffy::Dimension::from_percent(1.0),
      },
      ..Default::default()
    };
    let content_id = self.base.new_leaf_default(
      content_style,
      "WindowContent",
      |id, context, _shape, _actual_shape| {
        let content = WindowContent::new(
          id,
          context,
          buffer.clone(),
          Arc::downgrade(&mocked_viewport),
        );
        TreeNode::WindowContent(content)
      },
    )?;

    let id = self.base.new_with_parent_default(
      parent_id,
      style,
      "Window",
      |id, context, _shape, actual_shape| {
        let window = Window::new(
          id,
          context,
          opts,
          actual_shape.size(),
          buffer.clone(),
          content_id,
        );
        TreeNode::Window(window)
      },
    )?;

    // Insert window content (leaf) to window, as its child.
    self.base.add_child(id, content_id)?;

    // Insert the correct viewport back to window content.
    let viewport = self.window(id).viewport();
    match self.node_mut(content_id).unwrap() {
      TreeNode::WindowContent(c) => c.set_viewport(Arc::downgrade(&viewport)),
      _ => unreachable!(),
    }

    Ok(id)
  }

  /// See [`Itree::bounded_insert`].
  pub fn bounded_insert(
    &mut self,
    parent_id: TreeNodeId,
    child_node: TreeNode,
  ) -> Option<TreeNode> {
    self._insert_guard(&child_node);
    self.base.bounded_insert(parent_id, child_node)
  }

  fn remove_guard(&mut self, id: TreeNodeId) {
    if self.cmdline_id == Some(id) {
      self.cmdline_id = None;
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
    self.base.move_child(id)
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
    self.base.reserved_move_position_to(id, x, y)
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
