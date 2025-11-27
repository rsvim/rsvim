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
use taffy::TaffyResult;
use taffy::TaffyTree;
use taffy::prelude::FromLength;
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
      let mut base = lotree.borrow_mut();
      let root_id = base.new_leaf(root_style)?;
      base.compute_layout(root_id, taffy::Size::MAX_CONTENT)?;
      root_id
    };

    let root = Root::new(lotree.clone(), root_id);
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
    self.lotree.borrow().parent(id).copied()
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
  pub fn iter(&self) -> TreeIter {
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

  pub fn cursor(&self) -> Option<&Cursor> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.nodes.contains_key(&cursor_id));
        let cursor_node = self.node(cursor_id).unwrap();
        debug_assert!(matches!(cursor_node, TreeNode::Cursor(_)));
        match cursor_node {
          TreeNode::Cursor(w) => {
            debug_assert_eq!(w.id(), cursor_id);
            Some(w)
          }
          _ => unreachable!(),
        }
      }
      None => None,
    }
  }

  pub fn cursor_mut(&mut self) -> Option<&mut Cursor> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.nodes.contains_key(&cursor_id));
        let cursor_node = self.node_mut(cursor_id).unwrap();
        debug_assert!(matches!(cursor_node, TreeNode::Cursor(_)));
        match cursor_node {
          TreeNode::Cursor(w) => {
            debug_assert_eq!(w.id(), cursor_id);
            Some(w)
          }
          _ => unreachable!(),
        }
      }
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

impl Tree {
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
      _ => { /* Skip */ }
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
        width: taffy::Dimension::auto(),
        height: taffy::Dimension::auto(),
      },
      ..Default::default()
    };
    let (window_id, content_id) = {
      let mut base = self.lotree.borrow_mut();
      let window_id = base.new_with_parent(window_style, parent_id)?;
      let content_id = base.new_with_parent(content_style, window_id)?;
      base.compute_layout(parent_id, taffy::Size::MAX_CONTENT)?;
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
      let mut base = self.lotree.borrow_mut();
      let cursor_id = base.new_with_parent(cursor_style, parent_id)?;
      base.compute_layout(parent_id, taffy::Size::MAX_CONTENT)?;
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
        height: taffy::Dimension::auto(),
      },
      ..Default::default()
    };
    let input_style = Style {
      display: taffy::Display::None,
      size: taffy::Size {
        width: taffy::Dimension::auto(),
        height: taffy::Dimension::auto(),
      },
      ..Default::default()
    };
    let message_style = Style {
      size: taffy::Size {
        width: taffy::Dimension::auto(),
        height: taffy::Dimension::auto(),
      },
      ..Default::default()
    };

    let (cmdline_id, indicator_id, input_id, message_id) = {
      let mut base = self.lotree.borrow_mut();
      let indicator_id = base.new_leaf(indicator_style)?;
      let input_id = base.new_leaf(input_style)?;
      let message_id = base.new_leaf(message_style)?;
      let cmdline_id = base.new_with_children(
        cmdline_style,
        &[indicator_id, input_id, message_id],
      )?;
      base.add_child(parent_id, cmdline_id)?;
      base.compute_layout(parent_id, taffy::Size::MAX_CONTENT)?;
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

    self.nodes.remove(&id)
  }

  /// Set window viewport, returns old viewport.
  pub fn set_window_viewport(
    &mut self,
    id: TreeNodeId,
    viewport: ViewportArc,
  ) -> ViewportArc {
    debug_assert!(self.window_ids.contains(&id));
    let window = self.window_mut(id).unwrap();
    let old = window.viewport();
    window.set_viewport(viewport.clone());
    let content_id = window.content_id();
    debug_assert!(self.nodes.contains_key(&id));
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
    id: TreeNodeId,
    cursor_viewport: CursorViewportArc,
  ) -> CursorViewportArc {
    debug_assert!(self.window_ids.contains(&id));
    let window = self.window_mut(id).unwrap();
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
    let cmdline = self.command_line_mut().unwrap();
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
    let cmdline = self.command_line_mut().unwrap();
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
    let cmdline = self.command_line_mut().unwrap();
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
    let lotree = lotree.borrow_mut();
    let old_parent_id = *lotree.parent(cursor_id).unwrap();
    debug_assert!(self.nodes.contains_key(&old_parent_id));
    debug_assert!(matches!(self.node(old_parent_id).unwrap(), TreeNode::WindowContent(_) | TreeNode::CommandLineInput(_)));
    match self.node_mut(old_parent_id).unwrap() {
      TreeNode::WindowContent(content) => {
        // Cursor is inside a window content widget.
        let old_window_id = *lotree.parent(old_parent_id).unwrap();
        debug_assert!(self.nodes.contains_key(&old_window_id));
        // If new parent is the same window.
        if old_window_id == parent_id {
          return None;
        }
        if let TreeNode::Window(window) = self.node_mut(old_window_id).unwrap() {
          window.clear_cursor_id();
        }
      }
      TreeNode::CommandLineInput(input) => {
        // Cursor is inside the command-line input widget.
        let old_cmdline_id = *lotree.parent(old_parent_id).unwrap();
        debug_assert!(self.nodes.contains_key(&old_cmdline_id));
        // If new parent is the same window.
        if old_cmdline_id == parent_id {
          return None;
        }
        if let TreeNode::CommandLine(cmdline) = self.node_mut(old_cmdline_id).unwrap() {
          cmdline.clear_cursor_id();
        }
      }
      _ => unreachable!(),
    }
        debug_assert!(self.nodes.contains_key(&parent_id));
        match self.node_mut(parent_id) => {

        }
    None
  }
}

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
    self.lotree.bounded_move_by(id, x, y)
  }

  /// Bounded move to position x(columns) and y(rows). This is simply a wrapper method on
  /// [`Itree::bounded_move_to`].
  pub fn bounded_move_to(
    &mut self,
    id: TreeNodeId,
    x: isize,
    y: isize,
  ) -> Option<IRect> {
    self.lotree.bounded_move_to(id, x, y)
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

  pub fn set_global_local_options(&mut self, options: WindowOptions) {
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
      if !node.visible() {
        continue;
      }
      node.draw(&mut canvas);
    }
  }
}
// Draw }
