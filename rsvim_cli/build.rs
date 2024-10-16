use rsvim_core::buf::Buffers;
use rsvim_core::cart::U16Size;
use rsvim_core::cli::CliOpt;
use rsvim_core::js::{JsRuntime, JsRuntimeOptions};
use rsvim_core::state::State;
use rsvim_core::ui::tree::Tree;

use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::channel;

fn main() {
  let options = JsRuntimeOptions::default();
  let startup_moment = Instant::now();
  let startup_unix_epoch = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis();
  let (js_runtime_send_to_master, _master_recv_from_js_runtime) = channel(10);
  let (_master_send_to_js_runtime, js_runtime_recv_from_master) = channel(10);
  let cli_opt = CliOpt::default();
  let runtime_path = Arc::new(RwLock::new(vec![]));
  let tree = Tree::to_arc(Tree::new(U16Size::new(0, 0)));
  let buffers = Buffers::to_arc(Buffers::new());
  let state = State::to_arc(State::default());

  let mut js_runtime = JsRuntime::new(
    options,
    startup_moment,
    startup_unix_epoch,
    js_runtime_send_to_master,
    js_runtime_recv_from_master,
    cli_opt,
    runtime_path,
    tree,
    buffers,
    state,
  );

  js_runtime.create_snapshot(Path::new("snapshot.bin"));
}
