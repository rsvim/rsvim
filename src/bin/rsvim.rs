//! The VIM editor reinvented in Rust+TypeScript.

use rsvim::cli::CliOpt;
use rsvim::error::IoResult;
use rsvim::evloop::EventLoop;
use rsvim::log;

use clap::Parser;
// use heed::types as heed_types;
// use heed::{byteorder, Database, EnvOpenOptions};
use tracing::debug;

fn main() -> IoResult<()> {
  log::init();
  let cli_opt = CliOpt::parse();
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

  // Explicitly create tokio runtime for the EventLoop.
  let evloop_tokio_runtime = tokio::runtime::Runtime::new()?;
  evloop_tokio_runtime.block_on(async {
    // Create event loop.
    let mut event_loop = EventLoop::new(cli_opt)?;

    // Initialize.
    event_loop.init_js_runtime()?;
    event_loop.init_tui()?;
    event_loop.init_input_files()?;

    // Run loop.
    event_loop.run().await?;

    // Shutdown.
    event_loop.shutdown_tui()
  })
}
