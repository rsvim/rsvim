//! The VIM editor reinvented in Rust+TypeScript.
//!
//! See [rsvim_core] for more details.

use rsvim_core::cli::{CliOptions, LONG_HELP, SHORT_HELP, VERSION};
use rsvim_core::evloop::EventLoop;
use rsvim_core::js::SnapshotData;
use rsvim_core::log;
use rsvim_core::prelude::*;

use std::sync::LazyLock;

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

fn main() -> IoResult<()> {
  log::init();

  let cli_opts = match CliOptions::from_env() {
    Ok(cli_opts) => cli_opts,
    Err(e) => {
      println!("error: {e}");
      println!();
      println!("For more information, try '--help'");
      std::process::exit(1);
    }
  };
  trace!("cli_opts:{:?}", cli_opts);

  if cli_opts.special_opts().version() {
    println!("{}", *VERSION);
    std::process::exit(0);
  }
  if cli_opts.special_opts().short_help() {
    println!("{}", *SHORT_HELP);
    std::process::exit(0);
  }
  if cli_opts.special_opts().long_help() {
    println!("{}", *LONG_HELP);
    std::process::exit(0);
  }

  // Explicitly create tokio runtime for the EventLoop.
  let evloop_tokio_runtime = tokio::runtime::Runtime::new()?;
  evloop_tokio_runtime.block_on(async {
    // Create event loop.
    let mut event_loop =
      EventLoop::new(cli_opts, SnapshotData::new(&RSVIM_SNAPSHOT))?;

    // Initialize.
    event_loop.initialize()?;

    // Run loop.
    let mut reader = crossterm::event::EventStream::new();
    event_loop.run(reader).await?;

    // Shutdown.
    event_loop.shutdown()
  })
}
