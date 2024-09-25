//! The VIM editor reinvented in Rust+TypeScript.

#![allow(unused_imports, dead_code)]

use rsvim::evloop::EventLoop;
use rsvim::glovar;
// use rsvim::js::{start as js_start, JsDataAccess};
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

// /// Initialize TUI.
// pub fn init_tui() -> VoidIoResult {
//   if !terminal::is_raw_mode_enabled()? {
//     terminal::enable_raw_mode()?;
//   }
//
//   let mut out = std::io::stdout();
//   execute!(
//     out,
//     terminal::EnterAlternateScreen,
//     terminal::Clear(terminal::ClearType::All),
//     EnableMouseCapture,
//     EnableFocusChange,
//   )?;
//
//   Ok(())
// }
//
// /// Shutdown TUI.
// pub fn shutdown_tui() -> VoidIoResult {
//   let mut out = std::io::stdout();
//   execute!(
//     out,
//     DisableMouseCapture,
//     DisableFocusChange,
//     terminal::LeaveAlternateScreen,
//   )?;
//
//   if terminal::is_raw_mode_enabled()? {
//     terminal::disable_raw_mode()?;
//   }
//
//   Ok(())
// }

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
  // let (js_send_to_evloop, evloop_recv_from_js) = channel(glovar::CHANNEL_BUF_SIZE());
  // let (evloop_send_to_js, js_recv_from_evloop) = channel(glovar::CHANNEL_BUF_SIZE());

  // Initialize EventLoop.
  let mut event_loop = EventLoop::new(cli_opt)?;

  // Explicitly create tokio runtime for the EventLoop.
  let evloop_tokio_runtime = tokio::runtime::Runtime::new()?;
  evloop_tokio_runtime.block_on(async {
    // Initialize
    event_loop.init_tui()?;
    event_loop.init_input_files()?;

    // Running loop
    event_loop.run().await?;

    // Shutdown
    event_loop.shutdown_tui()
  })
}
