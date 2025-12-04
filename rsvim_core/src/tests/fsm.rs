use crate::buf::BufferArc;
use crate::buf::BuffersManagerArc;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::BufferOptionsBuilder;
use crate::content::TextContents;
use crate::content::TextContentsArc;
use crate::prelude::*;
use crate::state::StateDataAccess;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::buf::make_buffers_manager;
use crate::tests::tree::make_tree_with_buffers;
use crate::ui::tree::TreeArc;
use crate::ui::widget::window::opt::WindowOptions;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use tokio::sync::mpsc::unbounded_channel;

pub fn make_fsm_context(
  terminal_size: U16Size,
  buffer_local_opts: BufferOptions,
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
  let buf = make_buffer_from_lines(terminal_size, buffer_local_opts, lines);
  let bufs = make_buffers_manager(buffer_local_opts, vec![buf.clone()]);
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

pub fn make_default_fsm_context(
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
  make_fsm_context(terminal_size, buf_opts, window_local_opts, lines)
}

pub fn make_fsm_context_with_cmdline(
  terminal_size: U16Size,
  buffer_local_opts: BufferOptions,
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
  use crate::tests::tree::make_tree_with_buffers_cmdline;

  let buf = make_buffer_from_lines(terminal_size, buffer_local_opts, lines);
  let bufs = make_buffers_manager(buffer_local_opts, vec![buf.clone()]);
  let contents = TextContents::to_arc(TextContents::new(terminal_size));
  let tree = make_tree_with_buffers_cmdline(
    terminal_size,
    window_local_opts,
    bufs.clone(),
    contents.clone(),
  );

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

pub fn make_default_fsm_context_with_cmdline(
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
  make_fsm_context_with_cmdline(
    terminal_size,
    buf_opts,
    window_local_opts,
    lines,
  )
}
