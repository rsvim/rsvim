//! The widget tree that manages all the widget components.

pub mod internal;

use crate::buf::BufferWk;
use crate::content::TextContentsWk;
use crate::inode_dispatcher;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::canvas::CursorStyle;
use crate::ui::widget::Widgetable;
use crate::ui::widget::cmdline::Cmdline;
use crate::ui::widget::cmdline::indicator::CmdlineIndicator;
use crate::ui::widget::cmdline::indicator::CmdlineIndicatorSymbol;
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
use itertools::Itertools;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use taffy::Style;
use taffy::TaffyResult;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;
use taffy::prelude::TaffyAuto;

pub type TreeNodeId = i32;

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum TreeNode {
  Panel(Panel),
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
  Panel,
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
  Panel,
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
  // The reference of all common tree node relationships & attributes.
  context: TreeContextRc,

  // Maps node ID => node.
  nodes: FoldMap<TreeNodeId, TreeNode>,

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

// Node {
impl Tree {
  /// Make a widget tree.
  ///
  /// NOTE: The root node is created along with the tree.
  pub fn new(canvas_size: U16Size) -> TaffyResult<Self> {
    let (id, context) = {
      let mut context = TreeContext::new();
      let style = Style {
        size: taffy::Size {
          width: taffy::Dimension::from_length(canvas_size.width()),
          height: taffy::Dimension::from_length(canvas_size.height()),
        },
        flex_direction: taffy::FlexDirection::Column,
        ..Default::default()
      };
      let id = context.new_leaf_default(style, "Panel")?;
      context.compute_layout()?;
      (id, context)
    };

    let context = TreeContext::to_rc(context);
    let root = Panel::new(id, Rc::downgrade(&context));
    let root = TreeNode::Panel(root);

    let mut nodes = FoldMap::new();
    nodes.insert(id, root);

    Ok(Tree {
      context,
      nodes,
      cursor_id: None,
      cmdline_id: None,
      window_ids: BTreeSet::new(),
      current_window_id: None,
      global_options: WindowGlobalOptionsBuilder::default().build().unwrap(),
      global_local_options: WindowOptionsBuilder::default().build().unwrap(),
    })
  }

  fn _internal_check(&self) {
    debug_assert_eq!(self.context.borrow().len(), self.nodes.len());
  }

  /// Nodes count, include the root node.
  pub fn len(&self) -> usize {
    self._internal_check();
    self.nodes.len()
  }

  /// Whether the tree is empty.
  pub fn is_empty(&self) -> bool {
    self._internal_check();
    self.nodes.is_empty()
  }

  pub fn context(&self) -> TreeContextRc {
    self.context.clone()
  }

  /// Root node ID.
  pub fn root_id(&self) -> TreeNodeId {
    self._internal_check();
    self.context.borrow().root()
  }

  /// Get the parent ID by a node `id`.
  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self._internal_check();
    self.context.borrow().parent(id)
  }

  /// Get the children IDs by a node `id`.
  pub fn children_ids(&self, id: TreeNodeId) -> Vec<TreeNodeId> {
    self._internal_check();
    self.context.borrow().children(id).unwrap_or_default()
  }

  /// Get the node struct by its `id`.
  pub fn node(&self, id: TreeNodeId) -> Option<&TreeNode> {
    self._internal_check();
    self.nodes.get(&id)
  }

  /// Get mutable node struct by its `id`.
  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut TreeNode> {
    self._internal_check();
    self.nodes.get_mut(&id)
  }

  pub fn iter(&self) -> TreeIter<'_> {
    TreeIter::new(self, Some(self.context.borrow().root()))
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

  // Show message widget, hide indicator/input widgets.
  pub fn cmdline_show_message(&mut self) {
    let mut context = self.context.borrow_mut();
  }

  // Show indicator/input widgets, hide message widget.
  pub fn cmdline_show_input(&mut self) {}
}
// Widget }

// Insert/Remove {
impl Tree {
  fn _insert_node(&mut self, id: TreeNodeId, node: TreeNode) {
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
    self.nodes.insert(id, node);
  }

  /// Create a window widget.
  pub fn new_window_with_parent(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    opts: WindowOptions,
    buffer: BufferWk,
  ) -> TaffyResult<TreeNodeId> {
    let (id, content_id, content_actual_shape) = {
      let mut context = self.context.borrow_mut();

      // window
      let id = context.new_with_parent_default(parent_id, style, "Window")?;
      // window content
      let content_style = Style {
        size: taffy::Size {
          width: taffy::Dimension::from_percent(1.0),
          height: taffy::Dimension::from_percent(1.0),
        },
        ..Default::default()
      };
      let content_id =
        context.new_with_parent_default(id, content_style, "WindowContent")?;

      context.compute_layout()?;

      let content_actual_shape =
        context.actual_shape(content_id).copied().unwrap();

      (id, content_id, content_actual_shape)
    };

    // window node
    let window = Window::new(
      id,
      Rc::downgrade(&self.context()),
      opts,
      buffer.clone(),
      content_id,
      &content_actual_shape.size(),
    );
    let viewport = window.viewport();
    let window = TreeNode::Window(window);
    self._insert_node(id, window);

    // window content node
    let content = WindowContent::new(
      content_id,
      Rc::downgrade(&self.context()),
      buffer.clone(),
      Arc::downgrade(&viewport),
    );
    let content = TreeNode::WindowContent(content);
    self._insert_node(content_id, content);

    Ok(id)
  }

  /// Create a cursor widget.
  pub fn new_cursor_with_parent(
    &mut self,
    parent_id: TreeNodeId,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> TaffyResult<TreeNodeId> {
    let id = {
      let mut context = self.context.borrow_mut();

      let cursor_style = Style {
        position: taffy::Position::Absolute,
        size: taffy::Size {
          width: taffy::Dimension::from_length(1_u16),
          height: taffy::Dimension::from_length(1_u16),
        },
        inset: taffy::Rect {
          left: taffy::LengthPercentageAuto::from_length(0_u16),
          top: taffy::LengthPercentageAuto::from_length(0_u16),
          right: taffy::LengthPercentageAuto::AUTO,
          bottom: taffy::LengthPercentageAuto::AUTO,
        },
        ..Default::default()
      };
      let id = context.new_with_parent(
        parent_id,
        cursor_style,
        DEFAULT_ZINDEX,
        TruncatePolicy::RESERVED,
        *DEFAULT_SHAPE,
        *DEFAULT_ACTUAL_SHAPE,
        "Cursor",
      )?;

      context.compute_layout()?;

      id
    };

    let cursor = Cursor::new(id, self.context(), blinking, hidden, style);
    let cursor = TreeNode::Cursor(cursor);
    self._insert_node(id, cursor);

    Ok(id)
  }

  /// Create a command-line widget.
  pub fn new_cmdline_with_parent(
    &mut self,
    parent_id: TreeNodeId,
    style: Style,
    indicator_symbol: CmdlineIndicatorSymbol,
    text_contents: TextContentsWk,
  ) -> TaffyResult<TreeNodeId> {
    let (
      id,
      input_panel_id,
      indicator_id,
      input_id,
      input_actual_shape,
      message_id,
      message_actual_shape,
    ) = {
      let mut context = self.context.borrow_mut();

      let indicator_style = Style {
        ..Default::default()
      };
      let input_style = Style {
        ..Default::default()
      };
      let input_panel_style = Style {
        display: taffy::Display::None, // taffy::Display::Grid,
        grid_template_columns: vec![
          taffy::prelude::length(1_u16),
          taffy::prelude::fr(1_u16),
        ],
        ..Default::default()
      };
      let message_style = Style {
        size: taffy::Size {
          width: taffy::Dimension::from_percent(1.0),
          height: taffy::Dimension::from_percent(1.0),
        },
        ..Default::default()
      };

      let id = context.new_with_parent_default(parent_id, style, "Cmdline")?;
      let input_panel_id = context.new_with_parent_default(
        parent_id,
        input_panel_style,
        "CmdlinePanel",
      )?;
      let indicator_id = context.new_with_parent_default(
        input_panel_id,
        indicator_style,
        "CmdlineIndicator",
      )?;
      let input_id = context.new_with_parent_default(
        input_panel_id,
        input_style,
        "CmdlineInput",
      )?;
      let message_id =
        context.new_with_parent_default(id, message_style, "CmdlineMessage")?;

      context.compute_layout()?;

      let input_actual_shape = context.actual_shape(input_id).copied().unwrap();
      let message_actual_shape =
        context.actual_shape(message_id).copied().unwrap();

      (
        id,
        input_panel_id,
        indicator_id,
        input_id,
        input_actual_shape,
        message_id,
        message_actual_shape,
      )
    };

    let cmdline = Cmdline::new(
      id,
      Rc::downgrade(&self.context()),
      text_contents.clone(),
      input_panel_id,
      indicator_id,
      input_id,
      &input_actual_shape.size(),
      message_id,
      &message_actual_shape.size(),
    );
    let input_viewport = cmdline.input_viewport();
    let message_viewport = cmdline.message_viewport();
    let cmdline = TreeNode::Cmdline(cmdline);
    self._insert_node(id, cmdline);

    let input_panel =
      Panel::new(input_panel_id, Rc::downgrade(&self.context()));
    let input_panel = TreeNode::Panel(input_panel);
    self._insert_node(input_panel_id, input_panel);

    let indicator = CmdlineIndicator::new(
      indicator_id,
      Rc::downgrade(&self.context()),
      indicator_symbol,
    );
    let indicator = TreeNode::CmdlineIndicator(indicator);
    self._insert_node(indicator_id, indicator);

    let input = CmdlineInput::new(
      input_id,
      Rc::downgrade(&self.context()),
      text_contents.clone(),
      Arc::downgrade(&input_viewport),
    );
    let input = TreeNode::CmdlineInput(input);
    self._insert_node(input_id, input);

    let message = CmdlineMessage::new(
      message_id,
      Rc::downgrade(&self.context()),
      text_contents,
      Arc::downgrade(&message_viewport),
    );
    let message = TreeNode::CmdlineMessage(message);
    self._insert_node(message_id, message);

    Ok(id)
  }

  fn _remove_node(&mut self, id: TreeNodeId) {
    if self.cmdline_id == Some(id) {
      self.cmdline_id = None;
    }

    self.window_ids.remove(&id);
    if self.current_window_id == Some(id) {
      self.current_window_id = self.window_ids.last().copied();
    }

    if self.cursor_id == Some(id) {
      self.cursor_id = None;
    }
  }
}
// Insert/Remove }

// Movement {
impl Tree {
  fn raw_move_position_by(
    context: &TreeContext,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> IRect {
    let shape = context.shape(id).unwrap();
    let pos: IPos = shape.min().into();
    Self::raw_move_position_to(context, id, pos.x() + x, pos.y() + y)
  }

  fn raw_move_position_to(
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
    Some(Self::raw_move_position_by(&ctx, id, final_x, final_y))
  }

  /// Similar to [reserved_move_position_by](Self::reserved_move_position_by),
  /// but moves with absolute position instead of relative.
  pub fn reserved_move_position_to(
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

    Some(Self::raw_move_position_to(
      &ctx,
      id,
      final_pos.x(),
      final_pos.y(),
    ))
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
    for node in self.iter() {
      // trace!("Draw tree:{:?}", node);
      if !node.enabled() {
        continue;
      }
      node.draw(&mut canvas);
    }
  }
}
// Draw }

// TreeIter {
#[derive(Debug)]
/// Iterate all the tree nodes in level-order.
///
/// For each node, it first visits the node itself, then visits all its
/// children. This is also the order of rendering the whole UI tree to the
/// terminal.
pub struct TreeIter<'a> {
  tree: &'a Tree,
  que: VecDeque<TreeNodeId>,
}

impl<'a> Iterator for TreeIter<'a> {
  type Item = &'a TreeNode;

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

impl<'a> TreeIter<'a> {
  pub fn new(tree: &'a Tree, start_id: Option<TreeNodeId>) -> Self {
    let mut que = VecDeque::new();
    if let Some(id) = start_id {
      que.push_back(id);
    }
    Self { tree, que }
  }
}
// TreeIter }
