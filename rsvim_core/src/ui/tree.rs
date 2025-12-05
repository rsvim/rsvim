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
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::command_line::indicator::CommandLineIndicator;
use crate::ui::widget::command_line::indicator::IndicatorSymbol;
use crate::ui::widget::command_line::input::CommandLineInput;
use crate::ui::widget::command_line::message::CommandLineMessage;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::root::Root;
use crate::ui::widget::window::Window;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::opt::WindowGlobalOptions;
use crate::ui::widget::window::opt::WindowGlobalOptionsBuilder;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use crate::widget_dispatcher;
pub use internal::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;
use std::sync::Arc;
use taffy::Style;
use taffy::TaffyError;
use taffy::TaffyResult;
use taffy::TaffyTree;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;
use taffy::prelude::TaffyAuto;
use taffy::prelude::TaffyMaxContent;

pub type TreeNodeId = i32;
pub type TaffyTreeRc = Rc<RefCell<TaffyTree>>;
pub type TaffyTreeWk = Weak<RefCell<TaffyTree>>;

#[derive(Debug, Clone)]
/// The value holder for each widget.
pub enum TreeNode {
  Root(Root),
  Cursor(Cursor),
  Window(Window),
  WindowContent(WindowContent),
  CommandLine(CommandLine),
  CommandLineIndicator(CommandLineIndicator),
  CommandLineInput(CommandLineInput),
  CommandLineMessage(CommandLineMessage),
}

inode_dispatcher!(
  TreeNode,
  Root,
  Cursor,
  Window,
  WindowContent,
  CommandLine,
  CommandLineIndicator,
  CommandLineInput,
  CommandLineMessage
);

widget_dispatcher!(
  TreeNode,
  Root,
  Cursor,
  Window,
  WindowContent,
  CommandLine,
  CommandLineIndicator,
  CommandLineInput,
  CommandLineMessage
);

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
/// Taffy implements CSS layout algorithms, they are just right to laying out
/// Rsvim UI widgets as well. But layout just tells a node where it should be
/// rendering, it is still need to implement the rendering method by itself.
///
/// # Ownership
///
/// Parent owns its children:
///
/// - Children will be destroyed when their parent is.
/// - Children are displayed inside their parent's geometric shape, clipped by
///   boundaries. While the size of each node can be logically infinite on the
///   imaginary canvas.
/// - The `visible` (or `enabled`) attributes of a child is implicitly
///   inherited from it's parent, unless they're explicitly been set.
///
/// # Priority
///
/// Children have higher priority than their parent to both display and process
/// input events:
///
/// - Children are always displayed on top of their parent, and has higher
///   priority to process a user's input event when the event occurs within the
///   shape of the child. The event will fallback to their parent if the child
///   doesn't process it.
/// - For children that shade each other, the one with higher z-index has
///   higher priority to display and process the input events.
pub struct Tree {
  // Internal tree.
  lotree: ItreeRc,

  // Tree nodes.
  nodes: FoldMap<TreeNodeId, TreeNode>,

  // Root node ID.
  root_id: TreeNodeId,

  // CommandLine node ID.
  command_line_id: Option<TreeNodeId>,

  // Cursor node ID.
  cursor_id: Option<TreeNodeId>,

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

// Node {
impl Tree {
  /// Make UI tree.
  pub fn new(canvas_size: U16Size) -> TaffyResult<Self> {
    let lotree = Itree::to_rc(Itree::new());

    let root_style = Style {
      size: taffy::Size {
        width: taffy::Dimension::from_length(canvas_size.width()),
        height: taffy::Dimension::from_length(canvas_size.height()),
      },
      flex_direction: taffy::FlexDirection::Column,
      ..Default::default()
    };
    let root_id = {
      let mut lotree = lotree.borrow_mut();
      let root_id = lotree.new_leaf(root_style)?;
      lotree.compute_layout(root_id, taffy::Size::MAX_CONTENT)?;
      root_id
    };

    let root = Root::new(Rc::downgrade(&lotree), root_id);
    let root_node = TreeNode::Root(root);
    let mut nodes = FoldMap::new();
    nodes.insert(root_id, root_node);

    Ok(Tree {
      lotree,
      nodes,
      root_id,
      command_line_id: None,
      cursor_id: None,
      window_ids: BTreeSet::new(),
      current_window_id: None,
      global_options: WindowGlobalOptionsBuilder::default().build().unwrap(),
      global_local_options: WindowOptionsBuilder::default().build().unwrap(),
    })
  }

  pub fn lotree(&self) -> ItreeRc {
    self.lotree.clone()
  }

  /// Root node ID.
  pub fn root_id(&self) -> TreeNodeId {
    self.root_id
  }

  /// Get the parent ID by a node `id`.
  pub fn parent_id(&self, id: TreeNodeId) -> Option<TreeNodeId> {
    self.lotree.borrow().parent(id)
  }

  /// Get the children IDs by a node `id`.
  pub fn children_ids(&self, id: TreeNodeId) -> TaffyResult<Vec<TreeNodeId>> {
    self.lotree.borrow().children(id)
  }

  /// Get the node struct by its `id`.
  pub fn node(&self, id: TreeNodeId) -> Option<&TreeNode> {
    self.nodes.get(&id)
  }

  /// Get mutable node struct by its `id`.
  pub fn node_mut(&mut self, id: TreeNodeId) -> Option<&mut TreeNode> {
    self.nodes.get_mut(&id)
  }

  /// See [`Itree::iter`].
  pub fn iter<'s>(&'s self) -> TreeIter<'s> {
    TreeIter::new(self, Some(self.root_id))
  }

  // /// See [`Itree::iter_mut`].
  // pub fn iter_mut(&mut self) -> TreeIterMut {
  //   self.base.iter_mut()
  // }

  /// Get command-line node ID.
  pub fn command_line_id(&self) -> Option<TreeNodeId> {
    self.command_line_id
  }

  /// Get cursor node ID.
  pub fn cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id
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
// Node }

// Widget {
impl Tree {
  /// Window widget.
  pub fn window(&self, window_id: TreeNodeId) -> Option<&Window> {
    match self.node(window_id)? {
      TreeNode::Window(window) => {
        debug_assert_eq!(window.id(), window_id);
        Some(window)
      }
      _ => None,
    }
  }

  /// Mutable window widget.
  pub fn window_mut(&mut self, window_id: TreeNodeId) -> Option<&mut Window> {
    match self.node_mut(window_id)? {
      TreeNode::Window(window) => {
        debug_assert_eq!(window.id(), window_id);
        Some(window)
      }
      _ => None,
    }
  }

  /// Window content widget.
  pub fn window_content(
    &self,
    window_id: TreeNodeId,
  ) -> Option<&WindowContent> {
    let content_id = match self.node(window_id)? {
      TreeNode::Window(window) => window.content_id(),
      _ => return None,
    };
    match self.node(content_id)? {
      TreeNode::WindowContent(content) => Some(content),
      _ => None,
    }
  }

  /// Mutable window content widget.
  pub fn window_content_mut(
    &mut self,
    window_id: TreeNodeId,
  ) -> Option<&mut WindowContent> {
    let content_id = match self.node(window_id)? {
      TreeNode::Window(window) => window.content_id(),
      _ => return None,
    };
    match self.node_mut(content_id)? {
      TreeNode::WindowContent(content) => Some(content),
      _ => None,
    }
  }

  // Current window widget.
  pub fn current_window(&self) -> Option<&Window> {
    self.window(self.current_window_id?)
  }

  // Mutable current window widget.
  pub fn current_window_mut(&mut self) -> Option<&mut Window> {
    self.window_mut(self.current_window_id?)
  }

  pub fn cursor(&self) -> Option<&Cursor> {
    if cfg!(debug_assertions)
      && let TreeNode::Cursor(cursor) = self.node(self.cursor_id?)?
    {
      debug_assert_eq!(Some(cursor.id()), self.cursor_id);
    }
    match self.node(self.cursor_id?)? {
      TreeNode::Cursor(cursor) => Some(cursor),
      _ => None,
    }
  }

  pub fn cursor_mut(&mut self) -> Option<&mut Cursor> {
    if cfg!(debug_assertions)
      && let TreeNode::Cursor(cursor) = self.node(self.cursor_id?)?
    {
      debug_assert_eq!(Some(cursor.id()), self.cursor_id);
    }
    match self.node_mut(self.cursor_id?)? {
      TreeNode::Cursor(cursor) => Some(cursor),
      _ => None,
    }
  }

  // Command-line widget.
  pub fn cmdline(&self) -> Option<&CommandLine> {
    if cfg!(debug_assertions)
      && let TreeNode::CommandLine(cmdline) =
        self.node(self.command_line_id?)?
    {
      debug_assert_eq!(Some(cmdline.id()), self.command_line_id);
    }
    match self.node(self.command_line_id?)? {
      TreeNode::CommandLine(cmdline) => Some(cmdline),
      _ => None,
    }
  }

  // Mutable command-line widget.
  pub fn cmdline_mut(&mut self) -> Option<&mut CommandLine> {
    if cfg!(debug_assertions)
      && let TreeNode::CommandLine(cmdline) =
        self.node(self.command_line_id?)?
    {
      debug_assert_eq!(Some(cmdline.id()), self.command_line_id);
    }
    match self.node_mut(self.command_line_id?)? {
      TreeNode::CommandLine(cmdline) => Some(cmdline),
      _ => None,
    }
  }

  /// Command-line input widget.
  pub fn cmdline_input(&self) -> Option<&CommandLineInput> {
    let input_id = self.cmdline()?.input_id();
    match self.node(input_id)? {
      TreeNode::CommandLineInput(input) => Some(input),
      _ => None,
    }
  }

  /// Mutable command-line input widget.
  pub fn cmdline_input_mut(&mut self) -> Option<&mut CommandLineInput> {
    let input_id = self.cmdline()?.input_id();
    match self.node_mut(input_id)? {
      TreeNode::CommandLineInput(input) => Some(input),
      _ => None,
    }
  }

  /// Command-line message widget.
  pub fn cmdline_message(&self) -> Option<&CommandLineMessage> {
    let message_id = self.cmdline()?.message_id();
    match self.node(message_id)? {
      TreeNode::CommandLineMessage(message) => Some(message),
      _ => None,
    }
  }

  /// Mutable command-line message widget.
  pub fn cmdline_message_mut(&mut self) -> Option<&mut CommandLineMessage> {
    let message_id = self.cmdline()?.message_id();
    match self.node_mut(message_id)? {
      TreeNode::CommandLineMessage(message) => Some(message),
      _ => None,
    }
  }

  /// Command-line indicator widget.
  pub fn cmdline_indicator(&self) -> Option<&CommandLineIndicator> {
    let indicator_id = self.cmdline()?.indicator_id();
    match self.node(indicator_id)? {
      TreeNode::CommandLineIndicator(indicator) => Some(indicator),
      _ => None,
    }
  }

  /// Mutable command-line indicator widget.
  pub fn cmdline_indicator_mut(&mut self) -> Option<&mut CommandLineIndicator> {
    let indicator_id = self.cmdline()?.indicator_id();
    match self.node_mut(indicator_id)? {
      TreeNode::CommandLineIndicator(indicator) => Some(indicator),
      _ => None,
    }
  }
}
// Widget }

impl Tree {
  fn compute_layout(&self, lotree: &mut Itree) -> TaffyResult<()> {
    lotree.compute_layout(self.root_id(), taffy::Size::MAX_CONTENT)
  }

  fn _insert(&mut self, child_node: TreeNode) {
    // guard
    match &child_node {
      TreeNode::CommandLine(cmdline) => {
        // When insert command-line widget, update `command_line_id`.
        self.command_line_id = Some(cmdline.id());
      }
      TreeNode::Window(window) => {
        // When insert window widget, update `window_ids`.
        self.window_ids.insert(window.id());
      }
      TreeNode::Cursor(cursor) => {
        self.cursor_id = Some(cursor.id());
      }
      _ => {}
    }

    self.nodes.insert(child_node.id(), child_node);
  }

  /// Create new window node, and insert it as a child to the provided parent_id.
  pub fn add_new_window(
    &mut self,
    parent_id: TreeNodeId,
    window_style: Style,
    window_opts: WindowOptions,
    buffer: BufferWk,
  ) -> TaffyResult<TreeNodeId> {
    let content_style = Style {
      size: taffy::Size {
        width: taffy::Dimension::from_percent(1.0),
        height: taffy::Dimension::from_percent(1.0),
      },
      ..Default::default()
    };
    let (window_id, content_id) = {
      let lotree = self.lotree.clone();
      let mut lotree = lotree.borrow_mut();
      let window_id = lotree.new_with_parent(window_style, parent_id)?;
      let content_id = lotree.new_with_parent(content_style, window_id)?;
      self.compute_layout(&mut lotree)?;

      // We don't allow zero-area widget.
      let window_actual_shape = lotree.actual_shape(window_id)?;
      let content_actual_shape = lotree.actual_shape(content_id)?;
      if window_actual_shape.size().is_zero()
        || content_actual_shape.size().is_zero()
      {
        return Err(TaffyError::InvalidInputNode(taffy::NodeId::from(0_u64)));
      }

      (window_id, content_id)
    };

    let window = Window::new(
      Rc::downgrade(&self.lotree()),
      window_id,
      window_opts,
      content_id,
      buffer.clone(),
    )?;
    let viewport = window.viewport();
    let window_node = TreeNode::Window(window);
    self._insert(window_node);

    let content = WindowContent::new(
      Rc::downgrade(&self.lotree()),
      content_id,
      buffer,
      Arc::downgrade(&viewport),
    );
    let content_node = TreeNode::WindowContent(content);
    self._insert(content_node);

    Ok(window_id)
  }

  /// Create new cursor node, and insert it as a child to the provided parent_id.
  pub fn add_new_cursor(
    &mut self,
    parent_id: TreeNodeId,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> TaffyResult<TreeNodeId> {
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

    let cursor_id = {
      let lotree = self.lotree.clone();
      let mut lotree = lotree.borrow_mut();
      let cursor_id = lotree.new_with_parent(cursor_style, parent_id)?;
      self.compute_layout(&mut lotree)?;
      cursor_id
    };

    let cursor = Cursor::new(
      Rc::downgrade(&self.lotree()),
      cursor_id,
      blinking,
      hidden,
      style,
    );
    let cursor_node = TreeNode::Cursor(cursor);
    self._insert(cursor_node);

    Ok(cursor_id)
  }

  /// Create new cmdline node, and insert it as a child to the provided parent_id.
  pub fn add_new_cmdline(
    &mut self,
    parent_id: TreeNodeId,
    cmdline_style: Style,
    indicator_symbol: IndicatorSymbol,
    text_contents: TextContentsWk,
  ) -> TaffyResult<TreeNodeId> {
    let indicator_style = Style {
      display: taffy::Display::None,
      size: taffy::Size {
        width: taffy::Dimension::from_length(1_u16),
        height: taffy::Dimension::from_percent(1.0),
      },
      ..Default::default()
    };
    let input_style = Style {
      display: taffy::Display::None,
      size: taffy::Size {
        width: taffy::Dimension::from_percent(1.0),
        height: taffy::Dimension::from_percent(1.0),
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

    let (cmdline_id, indicator_id, input_id, message_id) = {
      let lotree = self.lotree.clone();
      let mut lotree = lotree.borrow_mut();
      let indicator_id = lotree.new_leaf(indicator_style)?;
      let input_id = lotree.new_leaf(input_style)?;
      let message_id = lotree.new_leaf(message_style)?;
      let cmdline_id = lotree.new_with_children(
        cmdline_style,
        &[indicator_id, input_id, message_id],
      )?;
      lotree.add_child(parent_id, cmdline_id)?;
      self.compute_layout(&mut lotree)?;

      // We don't allow zero-area widget.
      let cmdline_actual_shape = lotree.actual_shape(cmdline_id)?;
      let input_actual_shape = lotree.actual_shape(input_id)?;
      let message_actual_shape = lotree.actual_shape(message_id)?;
      if cmdline_actual_shape.size().is_zero()
        || input_actual_shape.size().is_zero()
        || message_actual_shape.size().is_zero()
      {
        return Err(TaffyError::InvalidInputNode(taffy::NodeId::from(0_u64)));
      }

      (cmdline_id, indicator_id, input_id, message_id)
    };

    let cmdline = CommandLine::new(
      Rc::downgrade(&self.lotree()),
      cmdline_id,
      indicator_id,
      input_id,
      message_id,
      text_contents.clone(),
    )?;
    let input_viewport = cmdline.input_viewport();
    let message_viewport = cmdline.message_viewport();

    let cmdline_node = TreeNode::CommandLine(cmdline);
    self._insert(cmdline_node);

    let indicator = CommandLineIndicator::new(
      Rc::downgrade(&self.lotree()),
      indicator_id,
      indicator_symbol,
    );
    let indicator_node = TreeNode::CommandLineIndicator(indicator);
    self._insert(indicator_node);

    let input = CommandLineInput::new(
      Rc::downgrade(&self.lotree()),
      input_id,
      text_contents.clone(),
      Arc::downgrade(&input_viewport),
    );
    let input_node = TreeNode::CommandLineInput(input);
    self._insert(input_node);

    let message = CommandLineMessage::new(
      Rc::downgrade(&self.lotree()),
      message_id,
      text_contents,
      Arc::downgrade(&message_viewport),
    );
    let message_node = TreeNode::CommandLineMessage(message);
    self._insert(message_node);

    Ok(cmdline_id)
  }

  fn _remove(&mut self, id: TreeNodeId) -> Option<TreeNode> {
    // guard
    if self.command_line_id == Some(id) {
      self.command_line_id = None;
    }
    self.window_ids.remove(&id);
    if self.current_window_id == Some(id)
      && let Some(last_window_id) = self.window_ids.last()
    {
      self.current_window_id = Some(*last_window_id);
    }
    if self.cursor_id == Some(id) {
      self.cursor_id = None;
    }

    self.nodes.remove(&id)
  }

  /// Set window viewport, returns old viewport.
  pub fn set_window_viewport(
    &mut self,
    window_id: TreeNodeId,
    viewport: ViewportArc,
  ) -> ViewportArc {
    debug_assert!(self.window_ids.contains(&window_id));
    let window = self.window_mut(window_id).unwrap();
    let old = window.viewport();
    window.set_viewport(viewport.clone());
    let content_id = window.content_id();
    debug_assert!(self.nodes.contains_key(&window_id));
    let content_node = self.node_mut(content_id).unwrap();
    debug_assert!(matches!(content_node, TreeNode::WindowContent(_)));
    match content_node {
      TreeNode::WindowContent(content) => {
        content.set_viewport(Arc::downgrade(&viewport))
      }
      _ => unreachable!(),
    }
    old
  }

  /// Set window cursor_viewport, returns old cursor_viewport.
  pub fn set_window_cursor_viewport(
    &mut self,
    window_id: TreeNodeId,
    cursor_viewport: CursorViewportArc,
  ) -> CursorViewportArc {
    debug_assert!(self.window_ids.contains(&window_id));
    let window = self.window_mut(window_id).unwrap();
    let old = window.cursor_viewport();
    window.set_cursor_viewport(cursor_viewport.clone());
    old
  }

  /// Set command-line input viewport, returns old viewport.
  pub fn set_cmdline_input_viewport(
    &mut self,
    viewport: ViewportArc,
  ) -> ViewportArc {
    debug_assert!(self.command_line_id.is_some());
    let cmdline = self.cmdline_mut().unwrap();
    let old = cmdline.input_viewport();
    cmdline.set_input_viewport(viewport.clone());
    let input_id = cmdline.input_id();
    debug_assert!(self.nodes.contains_key(&input_id));
    let input_node = self.node_mut(input_id).unwrap();
    debug_assert!(matches!(input_node, TreeNode::CommandLineInput(_)));
    match input_node {
      TreeNode::CommandLineInput(input) => {
        input.set_viewport(Arc::downgrade(&viewport))
      }
      _ => unreachable!(),
    }
    old
  }

  /// Set command-line input cursor_viewport, returns old cursor_viewport.
  pub fn set_cmdline_input_cursor_viewport(
    &mut self,
    cursor_viewport: CursorViewportArc,
  ) -> CursorViewportArc {
    debug_assert!(self.command_line_id.is_some());
    let cmdline = self.cmdline_mut().unwrap();
    let old = cmdline.input_cursor_viewport();
    cmdline.set_input_cursor_viewport(cursor_viewport.clone());
    old
  }

  /// Set command-line message viewport, returns old viewport.
  pub fn set_cmdline_message_viewport(
    &mut self,
    viewport: ViewportArc,
  ) -> ViewportArc {
    debug_assert!(self.command_line_id.is_some());
    let cmdline = self.cmdline_mut().unwrap();
    let old = cmdline.message_viewport();
    cmdline.set_message_viewport(viewport.clone());
    let message_id = cmdline.message_id();
    debug_assert!(self.nodes.contains_key(&message_id));
    let input_node = self.node_mut(message_id).unwrap();
    debug_assert!(matches!(input_node, TreeNode::CommandLineMessage(_)));
    match input_node {
      TreeNode::CommandLineMessage(message) => {
        message.set_viewport(Arc::downgrade(&viewport))
      }
      _ => unreachable!(),
    }
    old
  }

  fn _cmdline_toggle_input(&mut self, show_input: bool) -> TaffyResult<()> {
    let input_id = self.cmdline().unwrap().input_id();
    let indicator_id = self.cmdline().unwrap().indicator_id();
    let message_id = self.cmdline().unwrap().message_id();

    let lotree = self.lotree.clone();
    let mut lotree = lotree.borrow_mut();
    let mut input_style = lotree.style(input_id)?.clone();
    let mut indicator_style = lotree.style(indicator_id)?.clone();
    let mut message_style = lotree.style(message_id)?.clone();

    if show_input {
      input_style.display = taffy::Display::Flex;
      indicator_style.display = taffy::Display::Flex;
      message_style.display = taffy::Display::None;
    } else {
      input_style.display = taffy::Display::None;
      indicator_style.display = taffy::Display::None;
      message_style.display = taffy::Display::Flex;
    }

    lotree.set_style(input_id, input_style)?;
    lotree.set_style(indicator_id, indicator_style)?;
    lotree.set_style(message_id, message_style)?;
    Ok(())
  }

  /// Show input/indicator widget, hide message widget in command-line.
  pub fn cmdline_show_input(&mut self) -> TaffyResult<()> {
    self._cmdline_toggle_input(true)
  }

  /// Show message widget, hide input/indicator widget in command-line.
  pub fn cmdline_show_message(&mut self) -> TaffyResult<()> {
    self._cmdline_toggle_input(false)
  }

  /// Jump cursor to a new parent widget.
  ///
  /// Cursor's parent widget must be either a Window or a CommandLine. Here we
  /// only allow window ID or command-line ID as the parent ID.
  ///
  /// NOTE: While inside the internal implementations, cursor node's parent is
  /// either a window content node, or a command-line input node.
  ///
  /// # Returns
  /// It returns old parent widget ID if jumped successfully, otherwise it
  /// returns `None` if not jumped, e.g. it still stays in current widget.
  pub fn jump_cursor_to(
    &mut self,
    parent_id: TreeNodeId,
  ) -> Option<TreeNodeId> {
    let cursor_id = self.cursor_id.unwrap();
    let lotree = self.lotree.clone();
    let mut lotree = lotree.borrow_mut();
    let old_parent_id = lotree.parent(cursor_id).unwrap();
    debug_assert!(self.nodes.contains_key(&old_parent_id));
    debug_assert!(matches!(
      self.node(old_parent_id).unwrap(),
      TreeNode::WindowContent(_) | TreeNode::CommandLineInput(_)
    ));
    let old_parent_id = match self.node_mut(old_parent_id).unwrap() {
      TreeNode::WindowContent(_content) => {
        // Cursor is inside a window content widget.
        let old_window_id = lotree.parent(old_parent_id).unwrap();
        debug_assert_eq!(self.current_window_id, Some(old_window_id));
        debug_assert!(self.nodes.contains_key(&old_window_id));
        // If new parent is the same window.
        if old_window_id == parent_id {
          return None;
        }
        match self.node_mut(old_window_id).unwrap() {
          TreeNode::Window(_window) => {
            lotree.remove_child(old_parent_id, cursor_id).unwrap();
          }
          _ => unreachable!(),
        }
        old_window_id
      }
      TreeNode::CommandLineInput(_input) => {
        // Cursor is inside the command-line input widget.
        let old_cmdline_id = lotree.parent(old_parent_id).unwrap();
        debug_assert!(self.nodes.contains_key(&old_cmdline_id));
        // If new parent is the same window.
        if old_cmdline_id == parent_id {
          return None;
        }
        match self.node_mut(old_cmdline_id).unwrap() {
          TreeNode::CommandLine(_cmdline) => {
            lotree.remove_child(old_parent_id, cursor_id).unwrap();
          }
          _ => unreachable!(),
        }
        old_cmdline_id
      }
      _ => unreachable!(),
    };
    debug_assert!(self.nodes.contains_key(&parent_id));
    match self.node_mut(parent_id).unwrap() {
      TreeNode::Window(window) => {
        let content_id = window.content_id();
        lotree.add_child(content_id, cursor_id).unwrap();
      }
      TreeNode::CommandLine(cmdline) => {
        let cmdline_input_id = cmdline.input_id();
        lotree.add_child(cmdline_input_id, cursor_id).unwrap();
      }
      _ => unreachable!(),
    }

    self.compute_layout(&mut lotree)?;
    Some(old_parent_id)
  }

  /// Moves cursor to (x,y) position. X is column, Y is row.
  /// It returns new position/shape of the cursor if moved successfully,
  /// otherwise it returns `None` if doesn't move.
  ///
  /// NOTE: Cursor movement is bounded, it will never go out of its parent
  /// widget.
  pub fn move_cursor_to(&mut self, x: isize, y: isize) -> Option<U16Rect> {
    let cursor_id = self.cursor_id.unwrap();
    let lotree = self.lotree.clone();
    let mut lotree = lotree.borrow_mut();
    let parent_id = lotree.parent(cursor_id).unwrap();
    debug_assert!(self.nodes.contains_key(&parent_id));
    debug_assert!(matches!(
      self.node(parent_id).unwrap(),
      TreeNode::WindowContent(_) | TreeNode::CommandLineInput(_)
    ));
    let parent_actual_shape = lotree.actual_shape(parent_id).unwrap();
    let new_x =
      num_traits::clamp(x, 0, parent_actual_shape.size().width() as isize);
    let new_y =
      num_traits::clamp(y, 0, parent_actual_shape.size().height() as isize);
    let shape = lotree.shape(cursor_id).unwrap();
    let pos = shape.min();
    // If the new position is same with current position, no need to move.
    if pos.x == new_x && pos.y == new_y {
      return None;
    }

    let mut style = lotree.style(cursor_id).unwrap().clone();
    style.inset = taffy::Rect {
      left: taffy::LengthPercentageAuto::from_length(new_x as u16),
      top: taffy::LengthPercentageAuto::from_length(new_y as u16),
      right: taffy::LengthPercentageAuto::AUTO,
      bottom: taffy::LengthPercentageAuto::AUTO,
    };
    lotree.set_style(cursor_id, style).unwrap();
    self.compute_layout(&mut lotree).unwrap();
    Some(lotree.actual_shape(cursor_id).unwrap())
  }

  /// Moves cursor by (x,y) offset. X is column, Y is row.
  /// It returns new position/shape of the cursor if moved successfully,
  /// otherwise it returns `None` if doesn't move.
  ///
  /// NOTE: Cursor movement is bounded, it will never go out of its parent
  /// widget.
  pub fn move_cursor_by(&mut self, x: isize, y: isize) -> Option<U16Rect> {
    let (new_x, new_y) = {
      let cursor_id = self.cursor_id.unwrap();
      let lotree = self.lotree.clone();
      let lotree = lotree.borrow();
      let pos = lotree.actual_shape(cursor_id).unwrap().min();
      let new_x = num_traits::clamp((pos.x as isize) + x, 0, u16::MAX as isize);
      let new_y = num_traits::clamp((pos.y as isize) + y, 0, u16::MAX as isize);
      (new_x, new_y)
    };
    self.move_cursor_to(new_x, new_y)
  }
}

// Editable widgets {
//
// Editable is to describe a high-level widget, for example
// "window", "command-line", they are all "editable" because user can jump
// cursor into it, and start typing. For these high-level widgets, actual only
// 1 of the sub-components contains the cursor and allow accepting user typings.
//
// Since these are all editable widgets, they can share some common logic.
//
// For now we only have two kinds of high-level widgets:
// - Window, the actual editable component is "Window Content".
// - Command-line, the actual editable component is "Command-line Input".
impl Tree {
  pub fn editable_viewport(&self, id: TreeNodeId) -> ViewportArc {
    debug_assert!(self.nodes.contains_key(&id));
    let node = self.node(id).unwrap();
    debug_assert!(matches!(
      node,
      TreeNode::Window(_) | TreeNode::CommandLine(_)
    ));
    match node {
      TreeNode::Window(window) => window.viewport(),
      TreeNode::CommandLine(cmdline) => cmdline.input_viewport(),
      _ => unreachable!(),
    }
  }

  pub fn set_editable_viewport(
    &mut self,
    id: TreeNodeId,
    viewport: ViewportArc,
  ) {
    debug_assert!(self.nodes.contains_key(&id));
    let node = self.node_mut(id).unwrap();
    debug_assert!(matches!(
      node,
      TreeNode::Window(_) | TreeNode::CommandLine(_)
    ));
    let _ = match node {
      TreeNode::Window(_) => self.set_window_viewport(id, viewport),
      TreeNode::CommandLine(_) => self.set_cmdline_input_viewport(viewport),
      _ => unreachable!(),
    };
  }

  pub fn editable_cursor_viewport(&self, id: TreeNodeId) -> CursorViewportArc {
    debug_assert!(self.nodes.contains_key(&id));
    let node = self.node(id).unwrap();
    debug_assert!(matches!(
      node,
      TreeNode::Window(_) | TreeNode::CommandLine(_)
    ));
    match node {
      TreeNode::Window(window) => window.cursor_viewport(),
      TreeNode::CommandLine(cmdline) => cmdline.input_cursor_viewport(),
      _ => unreachable!(),
    }
  }

  pub fn set_editable_cursor_viewport(
    &mut self,
    id: TreeNodeId,
    cursor_viewport: CursorViewportArc,
  ) {
    debug_assert!(self.nodes.contains_key(&id));
    let node = self.node_mut(id).unwrap();
    debug_assert!(matches!(
      node,
      TreeNode::Window(_) | TreeNode::CommandLine(_)
    ));
    let _ = match node {
      TreeNode::Window(_) => {
        self.set_window_cursor_viewport(id, cursor_viewport)
      }
      TreeNode::CommandLine(_) => {
        self.set_cmdline_input_cursor_viewport(cursor_viewport)
      }
      _ => unreachable!(),
    };
  }

  pub fn editable_options(&self, id: TreeNodeId) -> &WindowOptions {
    debug_assert!(self.nodes.contains_key(&id));
    let node = self.node(id).unwrap();
    debug_assert!(matches!(
      node,
      TreeNode::Window(_) | TreeNode::CommandLine(_)
    ));
    match node {
      TreeNode::Window(window) => window.options(),
      TreeNode::CommandLine(cmdline) => cmdline.options(),
      _ => unreachable!(),
    }
  }

  pub fn editable_actual_shape(&self, id: TreeNodeId) -> U16Rect {
    debug_assert!(self.nodes.contains_key(&id));
    let node = self.node(id).unwrap();
    debug_assert!(matches!(
      node,
      TreeNode::Window(_) | TreeNode::CommandLine(_)
    ));
    match node {
      TreeNode::Window(_) => self.window_content(id).unwrap().actual_shape(),
      TreeNode::CommandLine(_) => self.cmdline_input().unwrap().actual_shape(),
      _ => unreachable!(),
    }
  }
}
// Editable widgets }

// Options {
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
// Options }

// Draw {
impl Tree {
  /// Draw the widget tree to canvas.
  pub fn draw(&self, canvas: CanvasArc) {
    let mut canvas = lock!(canvas);
    for node in self.iter() {
      // Node is invisible
      if !node.enabled() {
        continue;
      }
      node.draw(&mut canvas);
    }
  }
}
// Draw }
