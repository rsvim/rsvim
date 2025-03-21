//! The VIM editor reinvented in Rust+TypeScript.
//!
//! See [rsvim_core] for more details.

use rsvim_core::cli::CliOpt;
use rsvim_core::evloop::EventLoop;
use rsvim_core::js::{SnapshotData, v8_version};
use rsvim_core::log;
use rsvim_core::res::IoResult;

use clap::Parser;
use once_cell::sync::Lazy;
use tracing::trace;

static RSVIM_SNAPSHOT: Lazy<Box<[u8]>> = Lazy::new(|| {
  static COMPRESSED_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/RSVIM_SNAPSHOT.BIN"));
  zstd::bulk::decompress(
    &COMPRESSED_BYTES[4..],
    u32::from_le_bytes(COMPRESSED_BYTES[0..4].try_into().unwrap()) as usize,
  )
  .unwrap()
  .into_boxed_slice()
});

static CLI_VERSION: Lazy<String> = Lazy::new(|| {
  let cargo_toml_src = include_str!("../../../Cargo.toml");
  let cargo_toml_meta = cargo_toml_src.parse::<toml::Table>().unwrap();
  format!(
    "rsvim {} (v8 {})",
    cargo_toml_meta["workspace"]["package"]["version"]
      .as_str()
      .unwrap(),
    v8_version(),
  )
  .to_string()
});

fn main() -> IoResult<()> {
  log::init();
  let cli_opt = CliOpt::parse();
  trace!("cli_opt: {:?}", cli_opt);

  // Print version and exit
  if cli_opt.version() {
    println!("{}", CLI_VERSION.as_str());
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
    let mut event_loop = EventLoop::new(cli_opt, SnapshotData::new(&RSVIM_SNAPSHOT))?;

    // Initialize user config.
    event_loop.init_config()?;

    // Initialize terminal.
    event_loop.init_tui()?;

    // Initialize buffers and windows.
    event_loop.init_buffers()?;
    event_loop.init_windows()?;

    // Finish initialize terminal.
    event_loop.init_tui_done()?;

    // Run loop.
    event_loop.run().await?;

    // Shutdown.
    event_loop.shutdown_tui()
  })
}
