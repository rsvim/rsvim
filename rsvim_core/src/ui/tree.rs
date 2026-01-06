//! The widget tree that manages all the widget components.

pub mod internal;

use crate::buf::BufferWk;
use crate::content::TextContentsWk;
use crate::inode_dispatcher;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::canvas::CursorStyle;
use crate::ui::viewport::CursorViewportArc;
use crate::ui::viewport::ViewportArc;
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
use std::rc::Rc;
use std::sync::Arc;
use taffy::Style;
use taffy::TaffyResult;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;
use taffy::prelude::TaffyAuto;

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
/// The UI widget tree.
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

pub type TreeIter<'a> = ItreeIter<'a, TreeNode>;

arc_mutex_ptr!(Tree);

// Node {
impl Tree {
  /// Make a widget tree.
  ///
  /// NOTE: The root node is created along with the tree.
  pub fn new(style: Style) -> TaffyResult<Self> {
    let mut base = Itree::new();

    let id = {
      let context = base.context();
      let mut context = context.borrow_mut();
      let id = context.new_leaf_default(style, "Root")?;
      context.compute_layout(id)?;
      id
    };

    let root = Panel::new(id, Rc::downgrade(&base.context()));
    let root = TreeNode::Panel(root);
    base.nodes_mut().insert(id, root);

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

  pub fn context(&self) -> TreeContextRc {
    self.base.context()
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
    self.base.children_ids(id).unwrap_or_default()
  }

  /// Get the node struct by its `id`.
  pub fn node(&self, id: TreeNodeId) -> Option<&TreeNode> {
    self.base.nodes().get(&id)
  }

  /// Get mutable node struct by its `id`.
  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut TreeNode> {
    self.base.nodes_mut().get_mut(&id)
  }

  pub fn iter(&self) -> TreeIter<'_> {
    TreeIter::new(&self.base, Some(self.root_id()))
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
  pub fn cursor(&self) -> Option<&Cursor> {
    let cursor_id = self.cursor_id?;
    match self.node(cursor_id)? {
      TreeNode::Cursor(cursor) => {
        debug_assert_eq!(cursor.id(), cursor_id);
        Some(cursor)
      }
      _ => unreachable!(),
    }
  }

  /// Mutable cursor widget.
  pub fn cursor_mut(&mut self) -> Option<&mut Cursor> {
    let cursor_id = self.cursor_id?;
    match self.node_mut(cursor_id)? {
      TreeNode::Cursor(cursor) => {
        debug_assert_eq!(cursor.id(), cursor_id);
        Some(cursor)
      }
      _ => unreachable!(),
    }
  }

  /// Window widget.
  pub fn window(&self, id: TreeNodeId) -> Option<&Window> {
    match self.node(id)? {
      TreeNode::Window(window) => {
        debug_assert_eq!(window.id(), id);
        Some(window)
      }
      _ => unreachable!(),
    }
  }

  /// Mutable window widget.
  pub fn window_mut(&mut self, id: TreeNodeId) -> Option<&mut Window> {
    match self.node_mut(id)? {
      TreeNode::Window(window) => {
        debug_assert_eq!(window.id(), id);
        Some(window)
      }
      _ => unreachable!(),
    }
  }

  /// Current window widget.
  pub fn current_window(&self) -> Option<&Window> {
    let current_window_id = self.current_window_id?;
    self.window(current_window_id)
  }

  /// Mutable current window widget.
  pub fn current_window_mut(&mut self) -> Option<&mut Window> {
    let current_window_id = self.current_window_id?;
    self.window_mut(current_window_id)
  }

  /// Command-line widget.
  pub fn cmdline(&self) -> Option<&Cmdline> {
    let cmdline_id = self.cmdline_id?;
    match self.node(cmdline_id)? {
      TreeNode::Cmdline(cursor) => {
        debug_assert_eq!(cursor.id(), cmdline_id);
        Some(cursor)
      }
      _ => unreachable!(),
    }
  }

  /// Command-line input widget.
  pub fn cmdline_input(&self) -> Option<&CmdlineInput> {
    let cmdline_id = self.cmdline_id?;
    match self.node(cmdline_id)? {
      TreeNode::Cmdline(cmdline) => {
        debug_assert_eq!(cmdline.id(), cmdline_id);
        let input_id = cmdline.input_id();
        match self.node(input_id)? {
          TreeNode::CmdlineInput(input) => Some(input),
          _ => unreachable!(),
        }
      }
      _ => unreachable!(),
    }
  }

  /// Command-line message widget.
  pub fn cmdline_message(&self) -> Option<&CmdlineMessage> {
    let cmdline_id = self.cmdline_id?;
    match self.node(cmdline_id)? {
      TreeNode::Cmdline(cmdline) => {
        debug_assert_eq!(cmdline.id(), cmdline_id);
        let message_id = cmdline.message_id();
        match self.node(message_id)? {
          TreeNode::CmdlineMessage(message) => Some(message),
          _ => unreachable!(),
        }
      }
      _ => unreachable!(),
    }
  }

  // Mutable command-line widget.
  pub fn cmdline_mut(&mut self) -> Option<&mut Cmdline> {
    let cmdline_id = self.cmdline_id?;
    match self.node_mut(cmdline_id)? {
      TreeNode::Cmdline(cmdline) => {
        debug_assert_eq!(cmdline.id(), cmdline_id);
        Some(cmdline)
      }
      _ => unreachable!(),
    }
  }

  fn _toggle_cmdline_input_or_message(
    &mut self,
    show_input: bool,
  ) -> TaffyResult<()> {
    let cmdline = self.cmdline().unwrap();
    let cmdline_id = cmdline.id();
    let input_panel_id = cmdline.input_panel_id();
    let message_id = cmdline.message_id();

    let context = self.base.context();
    let mut context = context.borrow_mut();
    let mut input_panel_style = context.style(input_panel_id)?.clone();
    let mut message_style = context.style(message_id)?.clone();

    debug_assert_eq!(
      input_panel_style.display,
      if show_input {
        taffy::Display::None
      } else {
        taffy::Display::Grid
      }
    );
    debug_assert_eq!(
      message_style.display,
      if show_input {
        taffy::Display::Grid
      } else {
        taffy::Display::None
      }
    );

    input_panel_style.display = if show_input {
      taffy::Display::Grid
    } else {
      taffy::Display::None
    };
    message_style.display = if show_input {
      taffy::Display::None
    } else {
      taffy::Display::Grid
    };

    context.set_style(input_panel_id, input_panel_style)?;
    context.set_style(message_id, message_style)?;
    context.compute_layout(cmdline_id)
  }

  // Show message widget, hide indicator/input widgets.
  pub fn cmdline_show_message(&mut self) -> TaffyResult<()> {
    self._toggle_cmdline_input_or_message(false)
  }

  // Show indicator/input widgets, hide message widget.
  pub fn cmdline_show_input(&mut self) -> TaffyResult<()> {
    self._toggle_cmdline_input_or_message(true)
  }
}
// Widget }

// Insert/Remove {
impl Tree {
  fn _insert_node(&mut self, id: TreeNodeId, node: TreeNode) {
    match &node {
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
    self.base.nodes_mut().insert(id, node);
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
      let context = self.base.context();
      let mut context = context.borrow_mut();

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

      let root_id = context.root();
      context.compute_layout(root_id)?;

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
      let context = self.base.context();
      let mut context = context.borrow_mut();

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

      let root_id = context.root();
      context.compute_layout(root_id)?;

      id
    };

    let cursor =
      Cursor::new(id, Rc::downgrade(&self.context()), blinking, hidden, style);
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
      let context = self.base.context();
      let mut context = context.borrow_mut();

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
        size: taffy::Size {
          width: taffy::prelude::percent(1.0),
          height: taffy::prelude::percent(1.0),
        },
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

      let root_id = context.root();
      context.compute_layout(root_id)?;

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
  pub fn reserved_move_cursor_position_to(
    &mut self,
    x: isize,
    y: isize,
  ) -> TaffyResult<()> {
    let context = self.base.context();
    let mut context = context.borrow_mut();

    let cursor_id = self.cursor_id.unwrap();
    let parent_id = self.parent_id(cursor_id).unwrap();
    let new_shape = self
      .base
      .move_position_to(&context, cursor_id, x, y, TruncatePolicy::RESERVED)
      .unwrap();
    let new_pos: IPos = new_shape.min().into();
    let mut style = context.style(cursor_id)?.clone();
    style.inset = taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(new_pos.x() as u16),
      top: taffy::LengthPercentageAuto::from_length(new_pos.y() as u16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    };
    context.set_style(cursor_id, style)?;
    context.compute_layout(parent_id)
  }

  pub fn reserved_move_cursor_position_by(
    &mut self,
    x: isize,
    y: isize,
  ) -> TaffyResult<()> {
    let context = self.base.context();
    let mut context = context.borrow_mut();

    let cursor_id = self.cursor_id.unwrap();
    let parent_id = self.parent_id(cursor_id).unwrap();
    let new_shape = self
      .base
      .move_position_by(&context, cursor_id, x, y, TruncatePolicy::RESERVED)
      .unwrap();
    let new_pos: IPos = new_shape.min().into();
    let mut style = context.style(cursor_id)?.clone();
    style.inset = taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(new_pos.x() as u16),
      top: taffy::LengthPercentageAuto::from_length(new_pos.y() as u16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    };
    context.set_style(cursor_id, style)?;
    context.compute_layout(parent_id)
  }
}
// Movement }

// Command-line {
impl Tree {
  pub fn set_cmdline_message_viewport(&mut self, viewport: ViewportArc) {
    let cmdline_id = self.cmdline_id().unwrap();
    match self.node_mut(cmdline_id).unwrap() {
      TreeNode::Cmdline(cmdline) => {
        cmdline.set_message_viewport(viewport.clone());
        let message_id = cmdline.message_id();
        match self.node_mut(message_id).unwrap() {
          TreeNode::CmdlineMessage(message) => {
            message.set_viewport(Arc::downgrade(&viewport))
          }
          _ => unreachable!(),
        }
      }
      _ => unreachable!(),
    }
  }

  pub fn set_cmdline_indicator_symbol(
    &mut self,
    symbol: CmdlineIndicatorSymbol,
  ) -> CmdlineIndicatorSymbol {
    let cmdline_indicator_id = self.cmdline().unwrap().indicator_id();
    match self.node_mut(cmdline_indicator_id).unwrap() {
      TreeNode::CmdlineIndicator(indicator) => {
        let old = indicator.symbol();
        indicator.set_symbol(symbol);
        old
      }
      _ => unreachable!(),
    }
  }
}
// Command-line }

// Editable {
impl Tree {
  /// Get viewport from editable widget.
  /// NOTE: Only window and cmdline input component are *editable* widgets.
  pub fn editable_viewport(&self, id: TreeNodeId) -> ViewportArc {
    match self.node(id).unwrap() {
      TreeNode::Window(window) => window.viewport(),
      TreeNode::Cmdline(cmdline) => cmdline.input_viewport(),
      _ => unreachable!(),
    }
  }

  pub fn set_editable_viewport(
    &mut self,
    id: TreeNodeId,
    viewport: ViewportArc,
  ) {
    match self.node_mut(id).unwrap() {
      TreeNode::Window(window) => {
        window.set_viewport(viewport.clone());
        let content_id = window.content_id();
        match self.node_mut(content_id).unwrap() {
          TreeNode::WindowContent(content) => {
            content.set_viewport(Arc::downgrade(&viewport))
          }
          _ => unreachable!(),
        }
      }
      TreeNode::Cmdline(cmdline) => {
        cmdline.set_input_viewport(viewport.clone());
        let input_id = cmdline.input_id();
        match self.node_mut(input_id).unwrap() {
          TreeNode::CmdlineInput(input) => {
            input.set_viewport(Arc::downgrade(&viewport))
          }
          _ => unreachable!(),
        }
      }
      _ => unreachable!(),
    }
  }

  pub fn editable_cursor_viewport(&self, id: TreeNodeId) -> CursorViewportArc {
    match self.node(id).unwrap() {
      TreeNode::Window(window) => window.cursor_viewport(),
      TreeNode::Cmdline(cmdline) => cmdline.input_cursor_viewport(),
      _ => unreachable!(),
    }
  }

  pub fn set_editable_cursor_viewport(
    &mut self,
    id: TreeNodeId,
    cursor_viewport: CursorViewportArc,
  ) {
    match self.node_mut(id).unwrap() {
      TreeNode::Window(window) => {
        window.set_cursor_viewport(cursor_viewport.clone())
      }
      TreeNode::Cmdline(cmdline) => {
        cmdline.set_input_cursor_viewport(cursor_viewport)
      }
      _ => unreachable!(),
    }
  }

  pub fn editable_options(&self, id: TreeNodeId) -> &WindowOptions {
    match self.node(id).unwrap() {
      TreeNode::Window(window) => window.options(),
      TreeNode::Cmdline(cmdline) => cmdline.options(),
      _ => unreachable!(),
    }
  }

  pub fn editable_actual_shape(&self, id: TreeNodeId) -> U16Rect {
    let editable_id = match self.node(id).unwrap() {
      TreeNode::Window(window) => window.content_id(),
      TreeNode::Cmdline(cmdline) => cmdline.input_id(),
      _ => unreachable!(),
    };
    self.node(editable_id).unwrap().actual_shape()
  }
}
// Editable }

// Global options {
impl Tree {
  pub fn global_options(&self) -> &WindowGlobalOptions {
    &self.global_options
  }

  pub fn global_options_mut(&mut self) -> &mut WindowGlobalOptions {
    &mut self.global_options
  }

  pub fn set_global_options(&mut self, options: WindowGlobalOptions) {
    self.global_options = options;
  }

  pub fn global_local_options(&self) -> &WindowOptions {
    &self.global_local_options
  }

  pub fn global_local_options_mut(&mut self) -> &mut WindowOptions {
    &mut self.global_local_options
  }

  pub fn set_global_local_options(&mut self, options: WindowOptions) {
    self.global_local_options = options;
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
      if node.enabled() {
        node.draw(&mut canvas);
      }
    }
  }
}
// Draw }
