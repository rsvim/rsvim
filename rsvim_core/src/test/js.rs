use crate::buf::BuffersManager;
use crate::cli::CliOpt;
use crate::content::TextContents;
use crate::js::{JsRuntime, JsRuntimeOptions};
use crate::prelude::*;
use crate::state::State;
use crate::ui::tree::Tree;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc::channel;

pub fn make_js_runtime() -> JsRuntime {
  let canvas_size = U16Size::new(10, 10);
  let (jsrt_tick_dispatcher, _jsrt_tick_queue) = channel(1);
  let (jsrt_to_mstr, _mstr_from_jsrt) = channel(*CHANNEL_BUF_SIZE);
  let (_mstr_to_jsrt, jsrt_from_mstr) = channel(*CHANNEL_BUF_SIZE);

  let cli_opt = CliOpt::new(false, vec![]);
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
    jsrt_to_mstr,
    jsrt_from_mstr,
    cli_opt.clone(),
    tree.clone(),
    buffers_manager.clone(),
    text_contents.clone(),
    state.clone(),
  )
}
