//! The VIM window.

use crate::buf::BufferWk;
use crate::cart::{IRect, U16Rect};
use crate::ui::canvas::Canvas;
use crate::ui::tree::internal::{InodeId, Inodeable, Itree};
use crate::ui::tree::GlobalOptions;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::root::WindowRootContainer;
use crate::ui::widget::Widgetable;

use regex::Regex;
use tracing::debug;

pub mod content;
pub mod root;

#[derive(Debug, Clone)]
/// Window options.
pub struct WindowOptions {
  /// The 'wrap' option, also known as 'line-wrap', default to `true`.
  /// See: <https://vimhelp.org/options.txt.html#%27wrap%27>.
  pub wrap: bool,

  /// The 'line-break' option, also known as 'word-wrap', default to `false`.
  /// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>.
  pub line_break: bool,

  /// The 'break-at' option, default to `" ^I!@*-+;:,./?"`.
  /// See: <https://vimhelp.org/options.txt.html#%27breakat%27>.
  /// NOTE: This option represents the regex pattern to break word for 'line-break'.
  break_at: String,

  // The build regex object for 'break_at'.
  break_at_regex: Regex,
}

impl Default for WindowOptions {
  fn default() -> Self {
    let break_at = String::from(" ^I!@*-+;:,./?");
    WindowOptions {
      wrap: true,
      line_break: false,
      break_at: break_at.clone(),
      break_at_regex: Regex::new(&break_at).unwrap(),
    }
  }
}

impl WindowOptions {
  pub fn break_at(&self) -> &String {
    &self.break_at
  }

  pub fn set_break_at(&mut self, value: String) {
    self.break_at = value.clone();
    self.break_at_regex = Regex::new(&value).unwrap();
  }

  pub fn break_at_regex(&self) -> &Regex {
    &self.break_at_regex
  }
}

#[derive(Debug, Clone)]
/// The VIM window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct Window {
  base: Itree<WindowNode>,

  // The Window content widget ID.
  content_id: InodeId,

  // Local window options.
  // By default these options will inherit from global options of UI.
  options: WindowOptions,
}

impl Window {
  pub fn new(shape: IRect, buffer: BufferWk, global_options: &GlobalOptions) -> Self {
    let window_root = WindowRootContainer::new(shape);
    let window_root_id = window_root.id();
    let window_root_node = WindowNode::WindowRootContainer(window_root);

    let mut base = Itree::new(window_root_node);

    let window_content = WindowContent::new(shape, buffer);
    let window_content_id = window_content.id();
    let window_content_node = WindowNode::WindowContent(window_content);

    base.bounded_insert(&window_root_id, window_content_node);

    Window {
      base,
      content_id: window_content_id,
      options: global_options.window_options.clone(),
    }
  }
}

impl Inodeable for Window {
  fn id(&self) -> InodeId {
    self.base.root_id()
  }

  fn depth(&self) -> &usize {
    self.base.node(&self.base.root_id()).unwrap().depth()
  }

  fn depth_mut(&mut self) -> &mut usize {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .depth_mut()
  }

  fn zindex(&self) -> &usize {
    self.base.node(&self.base.root_id()).unwrap().zindex()
  }

  fn zindex_mut(&mut self) -> &mut usize {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .zindex_mut()
  }

  fn shape(&self) -> &IRect {
    self.base.node(&self.base.root_id()).unwrap().shape()
  }

  fn shape_mut(&mut self) -> &mut IRect {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .shape_mut()
  }

  fn actual_shape(&self) -> &U16Rect {
    self.base.node(&self.base.root_id()).unwrap().actual_shape()
  }

  fn actual_shape_mut(&mut self) -> &mut U16Rect {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .actual_shape_mut()
  }

  fn enabled(&self) -> &bool {
    self.base.node(&self.base.root_id()).unwrap().enabled()
  }

  fn enabled_mut(&mut self) -> &mut bool {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .enabled_mut()
  }

  fn visible(&self) -> &bool {
    self.base.node(&self.base.root_id()).unwrap().visible()
  }

  fn visible_mut(&mut self) -> &mut bool {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .visible_mut()
  }
}

impl Widgetable for Window {
  fn draw(&mut self, canvas: &mut Canvas) {
    // Do nothing.
    for node in self.base.iter_mut() {
      debug!("draw node:{:?}", node);
      node.draw(canvas);
    }
  }
}

impl Window {
  pub fn options(&self) -> &WindowOptions {
    &self.options
  }

  pub fn options_mut(&mut self) -> &mut WindowOptions {
    &mut self.options
  }

  pub fn wrap(&self) -> bool {
    self.options.wrap
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.options.wrap = value;
  }

  pub fn line_break(&self) -> bool {
    self.options.line_break
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.options.line_break = value;
  }

  pub fn break_at(&self) -> &String {
    self.options.break_at()
  }

  pub fn set_break_at(&mut self, value: String) {
    self.options.set_break_at(value);
  }

  pub fn break_at_regex(&self) -> &Regex {
    self.options.break_at_regex()
  }
}

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum WindowNode {
  WindowRootContainer(WindowRootContainer),
  WindowContent(WindowContent),
}

macro_rules! window_node_generate_dispatch {
  ($self_name:ident,$method_name:ident) => {
    match $self_name {
      WindowNode::WindowRootContainer(n) => n.$method_name(),
      WindowNode::WindowContent(n) => n.$method_name(),
    }
  };
}

impl Inodeable for WindowNode {
  fn id(&self) -> InodeId {
    window_node_generate_dispatch!(self, id)
  }

  fn depth(&self) -> &usize {
    window_node_generate_dispatch!(self, depth)
  }

  fn depth_mut(&mut self) -> &mut usize {
    window_node_generate_dispatch!(self, depth_mut)
  }

  fn zindex(&self) -> &usize {
    window_node_generate_dispatch!(self, zindex)
  }

  fn zindex_mut(&mut self) -> &mut usize {
    window_node_generate_dispatch!(self, zindex_mut)
  }

  fn shape(&self) -> &IRect {
    window_node_generate_dispatch!(self, shape)
  }

  fn shape_mut(&mut self) -> &mut IRect {
    window_node_generate_dispatch!(self, shape_mut)
  }

  fn actual_shape(&self) -> &U16Rect {
    window_node_generate_dispatch!(self, actual_shape)
  }

  fn actual_shape_mut(&mut self) -> &mut U16Rect {
    window_node_generate_dispatch!(self, actual_shape_mut)
  }

  fn enabled(&self) -> &bool {
    window_node_generate_dispatch!(self, enabled)
  }

  fn enabled_mut(&mut self) -> &mut bool {
    window_node_generate_dispatch!(self, enabled_mut)
  }

  fn visible(&self) -> &bool {
    window_node_generate_dispatch!(self, visible)
  }

  fn visible_mut(&mut self) -> &mut bool {
    window_node_generate_dispatch!(self, visible_mut)
  }
}

impl Widgetable for WindowNode {
  /// Draw widget on the canvas.
  fn draw(&mut self, canvas: &mut Canvas) {
    match self {
      WindowNode::WindowRootContainer(w) => w.draw(canvas),
      WindowNode::WindowContent(w) => w.draw(canvas),
    }
  }
}
