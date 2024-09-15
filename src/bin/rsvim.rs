//! The VIM editor reinvented in Rust+TypeScript.

#![allow(unused_imports, dead_code)]

use rsvim::evloop::EventLoop;
use rsvim::result::VoidIoResult;
use rsvim::rt::{init_v8_platform, JsDataAccess, JsRuntime};
use rsvim::{cli, log};

use clap::Parser;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{execute, terminal};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
// use heed::types as heed_types;
// use heed::{byteorder, Database, EnvOpenOptions};
use tracing::debug;

/// Initialize TUI.
pub fn init_tui() -> VoidIoResult {
  if !terminal::is_raw_mode_enabled()? {
    terminal::enable_raw_mode()?;
  }

  let mut out = std::io::stdout();
  execute!(
    out,
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    EnableMouseCapture,
    EnableFocusChange,
  )?;

  Ok(())
}

/// Shutdown TUI.
pub fn shutdown_tui() -> VoidIoResult {
  let mut out = std::io::stdout();
  execute!(
    out,
    DisableMouseCapture,
    DisableFocusChange,
    terminal::LeaveAlternateScreen,
  )?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  Ok(())
}

#[tokio::main]
async fn main() -> VoidIoResult {
  log::init();
  let cli_opt = cli::CliOpt::parse();
  debug!("cli_opt: {:?}", cli_opt);
  let cpu_cores = if let Ok(n) = std::thread::available_parallelism() {
    n.get()
  } else {
    8_usize
  };
  debug!("CPU cores: {:?}", cpu_cores);

  // let dir = tempfile::tempdir().unwrap();
  // debug!("tempdir:{:?}", dir);
  // let env = unsafe { EnvOpenOptions::new().open(dir.path()).unwrap() };
  // let mut wtxn = env.write_txn().unwrap();
  // let db: Database<heed_types::Str, heed_types::U32<byteorder::NativeEndian>> =
  //   env.create_database(&mut wtxn, None).unwrap();
  // db.put(&mut wtxn, "seven", &7).unwrap();
  // wtxn.commit().unwrap();

  init_tui()?;

  let (js_send_to_evloop, evloop_recv_from_js) = unbounded_channel();
  let (evloop_send_to_js, js_recv_from_evloop) = unbounded_channel();

  // Event loop initialize
  let mut event_loop = EventLoop::new(cli_opt, evloop_send_to_js, evloop_recv_from_js)?;
  event_loop.init()?;

  // Js runtime initialize.
  //
  // Since rusty_v8 (for now) only support single thread mode, and the `Isolate` is not safe to be
  // sent between threads, here we allocate a single thread to run it. This is completely out of
  // tokio async runtime, and uses channel to communicate between V8 and the event loop.
  //
  // This is quite like a parent-child process relationship, js runtime thread can directly access
  // the EventLoop by simply acquire the RwLock.
  init_v8_platform();
  let mut js_runtime = JsRuntime::new(
    ".rsvim.js".to_string(),
    js_send_to_evloop,
    js_recv_from_evloop,
  );
  let data_access = JsDataAccess::new(
    event_loop.state.clone(),
    event_loop.tree.clone(),
    event_loop.buffers.clone(),
  );
  std::thread::spawn(move || {
    // Basically, this thread is simply running a single js/ts file, there are several tasks need
    // to complete:
    // 1. Resolve all the modules marked by `import` and `require` keywords, and recursively
    //    resolve the nested modules inside them.
    // 2. Update editor configurations and settings via the OPs.
    // 3. Bind callbacks (most interactives are triggered by callbacks) on the related Vim events,
    //    and schedule timeout/delay background jobs.
    let _ = js_runtime.start(data_access);

    // After loading user config is done, this thread is waiting for Event Loop to notify it to
    // exit. If the editor is quit before loading is done, then we need to insert some checks to
    // manually break config loading and exit this thread.
  });

  event_loop.run().await?;

  shutdown_tui()
}
