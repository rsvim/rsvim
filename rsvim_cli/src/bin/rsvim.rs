//! The VIM editor reinvented in Rust+TypeScript.
//!
//! See [rsvim_core] for more details.

use rsvim_core::cli::{
  CliOptions, LONG_HELP, RSVIM_BIN_NAME, RSVIM_PKG_VERSION, RSVIM_V8_VERSION,
  SHORT_HELP, VERSION,
};
use rsvim_core::evloop::EventLoop;
use rsvim_core::js::{SnapshotData, v8_version};
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

static RSVIM_VERSION: LazyLock<String> = LazyLock::new(|| {
  let pkg_version = concat!(
    concat!(
      concat!(concat!(env!("CARGO_PKG_VERSION"), "+"), env!("PROFILE")),
      "+"
    ),
    env!("OPT_LEVEL")
  );
  VERSION
    .replace(RSVIM_BIN_NAME, env!("CARGO_BIN_NAME"))
    .replace(RSVIM_PKG_VERSION, pkg_version)
    .replace(RSVIM_V8_VERSION, v8_version())
});

static RSVIM_SHORT_HELP: LazyLock<String> =
  LazyLock::new(|| SHORT_HELP.replace(RSVIM_BIN_NAME, env!("CARGO_BIN_NAME")));

static RSVIM_LONG_HELP: LazyLock<String> =
  LazyLock::new(|| LONG_HELP.replace(RSVIM_BIN_NAME, env!("CARGO_BIN_NAME")));

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
    println!("{}", *RSVIM_VERSION);
    std::process::exit(0);
  }
  if cli_opts.special_opts().short_help() {
    println!("{}", *RSVIM_SHORT_HELP);
    std::process::exit(0);
  }
  if cli_opts.special_opts().long_help() {
    println!("{}", *RSVIM_LONG_HELP);
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
    event_loop.run().await?;

    // Shutdown.
    event_loop.shutdown()
  })
}
