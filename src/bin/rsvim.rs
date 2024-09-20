//! The VIM editor reinvented in Rust+TypeScript.

#![allow(unused_imports, dead_code)]

use rsvim::evloop::EventLoop;
use rsvim::glovar;
use rsvim::js::{init_v8_platform, JsDataAccess, JsRuntime};
use rsvim::result::VoidIoResult;
use rsvim::{cli, log};

use clap::Parser;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{execute, terminal};
use tokio::sync::mpsc::{channel, Receiver, Sender};
// use heed::types as heed_types;
// use heed::{byteorder, Database, EnvOpenOptions};
use tracing::{debug, error};

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

fn main() -> VoidIoResult {
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

  // Two sender/receiver to send messages between js runtime and event loop in bidirections.
  let (js_send_to_evloop, evloop_recv_from_js) = channel(glovar::CHANNEL_BUF_SIZE());
  let (evloop_send_to_js, js_recv_from_evloop) = channel(glovar::CHANNEL_BUF_SIZE());

  // Initialize EventLoop.
  let mut event_loop = EventLoop::new(cli_opt, evloop_send_to_js, evloop_recv_from_js)?;

  // Initialize JavaScript runtime.
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
  let js_runtime_join_handle = std::thread::spawn(move || {
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

  // Explicitly create tokio runtime for the EventLoop.
  let evloop_rt = tokio::runtime::Runtime::new()?;
  let event_loop_result = evloop_rt.block_on(async {
    init_tui()?;

    // Move
    event_loop.init()?;

    event_loop.run().await?;

    shutdown_tui()
  });

  match js_runtime_join_handle.join() {
    Ok(_) => { /* Skip */ }
    Err(e) => error!("Failed to join Js runtime thread: {:?}", e),
  }

  event_loop_result
}
