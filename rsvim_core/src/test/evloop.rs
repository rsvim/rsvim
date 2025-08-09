use crate::buf::{BuffersManager, BuffersManagerArc};
use crate::cli::CliOptions;
use crate::content::{TextContents, TextContentsArc};
use crate::evloop::EventLoop;
use crate::js::msg::{
  self as jsmsg, EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage,
};
use crate::js::{JsRuntime, JsRuntimeOptions, SnapshotData};
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;

use msg::WorkerToMasterMessage;
#[cfg(test)]
use reader::mock_reader::MockReader;
use writer::{StdoutWritable, StdoutWriterValue};

#[cfg(test)]
use bitflags::bitflags_match;
use crossterm::event::{Event, EventStream};
#[cfg(test)]
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use futures::stream::StreamExt;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub fn make_event_loop() -> EventLoop {
  let canvas_size = U16Size::new(10, 10);
  let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
  let (jsrt_to_master, _master_from_jsrt) = channel(1);
  let (_master_to_jsrt, jsrt_from_master) = channel(1);

  let cli_opts = CliOptions::from_args(&vec![]).unwrap();
  let state = State::to_arc(State::new(jsrt_tick_dispatcher.clone()));

  let tree = Tree::to_arc(Tree::new(canvas_size));
  let buffers_manager = BuffersManager::to_arc(BuffersManager::new());
  let text_contents = TextContents::to_arc(TextContents::new(canvas_size));

  let startup_moment = Instant::now();
  let startup_unix_epoch = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis();

  JsRuntime::new_without_snapshot(
    JsRuntimeOptions::default(),
    startup_moment,
    startup_unix_epoch,
    jsrt_to_master,
    jsrt_from_master,
    cli_opts,
    tree.clone(),
    buffers_manager.clone(),
    text_contents.clone(),
    state.clone(),
  )
}
