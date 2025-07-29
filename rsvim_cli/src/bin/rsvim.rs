//! The VIM editor reinvented in Rust+TypeScript.
//!
//! See [rsvim_core] for more details.

use rsvim_core::cli::CliOpt;
use rsvim_core::evloop::EventLoop;
use rsvim_core::js::SnapshotData;
use rsvim_core::log;
use rsvim_core::prelude::*;

use clap::Parser;
use std::sync::LazyLock;
use tracing::trace;

static RSVIM_SNAPSHOT: LazyLock<Box<[u8]>> = LazyLock::new(|| {
  static COMPRESSED_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/RSVIM_SNAPSHOT.BIN"));
  zstd::bulk::decompress(
    &COMPRESSED_BYTES[4..],
    u32::from_le_bytes(COMPRESSED_BYTES[0..4].try_into().unwrap()) as usize,
  )
  .unwrap()
  .into_boxed_slice()
});

static RSVIM_VERSION: LazyLock<&str> = LazyLock::new(|| {
  static VERSION: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/RSVIM_VERSION.TXT"));
  VERSION
});

fn main() -> IoResult<()> {
  log::init();
  let cli_opt = CliOpt::parse();
  trace!("cli_opt: {:?}", cli_opt);

  // Print version and exit
  if cli_opt.version() {
    println!("{}", *RSVIM_VERSION);
    return Ok(());
  }

  // let dir = tempfile::tempdir().unwrap();
  // trace!("tempdir:{:?}", dir);
  // let env = unsafe { EnvOpenOptions::new().open(dir.path()).unwrap() };
  // let mut wtxn = env.write_txn().unwrap();
  // let db: Database<heed_types::Str, heed_types::U32<byteorder::NativeEndian>> =
  //   env.create_database(&mut wtxn, None).unwrap();
  // db.put(&mut wtxn, "seven", &7).unwrap();
  // wtxn.commit().unwrap();

  // Explicitly create tokio runtime for the EventLoop.
  let evloop_tokio_runtime = tokio::runtime::Runtime::new()?;
  evloop_tokio_runtime.block_on(async {
    // Create event loop.
    let mut event_loop =
      EventLoop::new(cli_opt, SnapshotData::new(&RSVIM_SNAPSHOT))?;

    // Initialize user config.
    event_loop.init_config()?;

    // Finish initialize terminal.
    event_loop.init_tui()?;

    // Initialize buffers and windows.
    event_loop.init_buffers()?;
    event_loop.init_windows()?;

    // Finish initialize terminal.
    event_loop.init_tui_complete()?;

    // Run loop.
    event_loop.run().await?;

    // Shutdown terminal raw mode.
    event_loop.shutdown_tui()
  })
}
