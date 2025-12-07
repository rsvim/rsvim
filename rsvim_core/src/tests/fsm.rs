use crate::buf::BufferArc;
use crate::buf::BuffersManagerArc;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::BufferOptionsBuilder;
use crate::buf::opt::FileFormatOption;
use crate::buf::text::Text;
use crate::content::TextContents;
use crate::content::TextContentsArc;
use crate::prelude::*;
use crate::state::StateDataAccess;
use crate::state::StateMachine;
use crate::state::Stateful;
use crate::state::fsm::NormalStateful;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::state::ops::cursor_ops;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::buf::make_buffers_manager;
use crate::tests::log::init as test_log_init;
use crate::tests::tree::make_tree_with_buffers;
use crate::tests::tree::make_tree_with_buffers_cmdline;
use crate::tests::viewport::assert_canvas;
use crate::tests::viewport::assert_viewport;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::tree::*;
use crate::ui::viewport::CursorViewport;
use crate::ui::viewport::CursorViewportArc;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::viewport::ViewportSearchDirection;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

pub fn make_tree(
  terminal_size: U16Size,
  window_local_opts: WindowOptions,
  lines: Vec<&str>,
) -> (
  Event,
  TreeArc,
  BuffersManagerArc,
  BufferArc,
  TextContentsArc,
  StateDataAccess,
) {
  let buf_opts = BufferOptionsBuilder::default().build().unwrap();
  let buf = make_buffer_from_lines(terminal_size, buf_opts, lines);
  let bufs = make_buffers_manager(buf_opts, vec![buf.clone()]);
  let tree =
    make_tree_with_buffers(terminal_size, window_local_opts, bufs.clone());
  let contents = TextContents::to_arc(TextContents::new(terminal_size));

  let key_event = KeyEvent::new_with_kind(
    KeyCode::Char('a'),
    KeyModifiers::empty(),
    KeyEventKind::Press,
  );
  let event = Event::Key(key_event);

  let (jsrt_forwarder_tx, _jsrt_forwarder_rx) = unbounded_channel();
  let (master_tx, _master_rx) = unbounded_channel();
  let data_access = StateDataAccess::new(
    tree.clone(),
    bufs.clone(),
    contents.clone(),
    master_tx,
    jsrt_forwarder_tx,
  );

  (event, tree, bufs, buf, contents, data_access)
}
