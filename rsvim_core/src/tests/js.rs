use crate::buf::BuffersManager;
use crate::cli::CliOptions;
use crate::content::TextContents;
use crate::js::JsRuntime;
use crate::js::JsRuntimeOptions;
use crate::js::command::ExCommandsManager;
use crate::prelude::*;
use crate::ui::tree::Tree;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use tokio::sync::mpsc::channel;

pub fn make_js_runtime() -> JsRuntime {
  let canvas_size = U16Size::new(10, 10);
  let (master_tx, _master_rx) = channel(1);
  let (_jsrt_tx, jsrt_rx) = channel(1);

  let cli_opts =
    CliOptions::from_args::<&Vec<std::ffi::OsString>>(&vec![]).unwrap();

  let tree = Tree::to_arc(Tree::new(canvas_size));
  let buffers_manager = BuffersManager::to_arc(BuffersManager::new());
  let text_contents = TextContents::to_arc(TextContents::new(canvas_size));
  let ex_commands_manager = ExCommandsManager::to_arc(ExCommandsManager::new());

  let startup_moment = Instant::now();
  let startup_unix_epoch = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis();

  JsRuntime::new_without_snapshot(
    JsRuntimeOptions::default(),
    startup_moment,
    startup_unix_epoch,
    master_tx,
    jsrt_rx,
    cli_opts,
    tree,
    buffers_manager,
    text_contents,
    ex_commands_manager,
  )
}
