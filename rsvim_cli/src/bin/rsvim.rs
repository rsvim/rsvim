//! The VIM editor reinvented in Rust+TypeScript.
//!
//! See [rsvim_core] for more details.

use rsvim_core::cli::CliOptions;
use rsvim_core::evloop::{EventLoop, EventLoopOptions};
use rsvim_core::js::{SnapshotData, v8_version};
use rsvim_core::log;
use rsvim_core::prelude::*;

use clap::Parser;
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
  let cli_opts = CliOptions::parse();
  trace!("cli_opt: {:?}", cli_opts);

  // Print version and exit
  if cli_opts.version() {
    let pkg_version = env!("CARGO_PKG_VERSION");
    println!("rsvim {} (v8 {})", pkg_version, v8_version());
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
    let event_loop_opts = EventLoopOptions::default();
    let mut event_loop = EventLoop::new(
      event_loop_opts,
      cli_opts,
      SnapshotData::new(&RSVIM_SNAPSHOT),
    )?;

    // Initialize.
    event_loop.initialize()?;

    // Run loop.
    event_loop.run().await?;

    // Shutdown.
    event_loop.shutdown()
  })
}
