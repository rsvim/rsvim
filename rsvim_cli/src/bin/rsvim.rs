//! The VIM editor reinvented in Rust+TypeScript.
//!
//! See [rsvim_core] for more details.

#[cfg(feature = "snmalloc")]
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

#[cfg(all(not(target_env = "msvc"), feature = "jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use rsvim_core::cli::CliOptions;
use rsvim_core::evloop::EventLoop;
use rsvim_core::js::SnapshotData;
use rsvim_core::log;
use rsvim_core::prelude::*;
use std::sync::LazyLock;

const RSVIM_BIN_NAME: &str = "{RSVIM_BIN_NAME}";
const RSVIM_PKG_VERSION: &str = "{RSVIM_PKG_VERSION}";

const RSVIM_SNAPSHOT: &[u8] =
  include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/RSVIM_SNAPSHOT.BIN"));

static RSVIM_VERSION: LazyLock<String> = LazyLock::new(|| {
  const VERSION: &str = "{RSVIM_BIN_NAME} {RSVIM_PKG_VERSION}";

  let pkg_version =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/RSVIM_VERSION.TXT"));
  VERSION
    .replace(RSVIM_BIN_NAME, env!("CARGO_BIN_NAME"))
    .replace(RSVIM_PKG_VERSION, pkg_version)
});

// --headless (experimental)  Run in headless mode without TUI
static RSVIM_SHORT_HELP: LazyLock<String> = LazyLock::new(|| {
  const SHORT_HELP: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/SHORT_HELP.TXT"));
  SHORT_HELP.replace(RSVIM_BIN_NAME, env!("CARGO_BIN_NAME"))
});

// --headless (experimental)
//     Run in headless mode without TUI. In this mode, rsvim doesn't enter
//     terminal's raw mode, it uses STDIN to receive javascript script, and
//     uses STDOUT, STDERR to print messages instead of rendering TUI. All
//     internal data structures (such as buffers, windows, command-line,
//     etc) and scripts/plugins will still be initialized
static RSVIM_LONG_HELP: LazyLock<String> = LazyLock::new(|| {
  const LONG_HELP: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/LONG_HELP.TXT"));
  LONG_HELP.replace(RSVIM_BIN_NAME, env!("CARGO_BIN_NAME"))
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
      EventLoop::new(cli_opts, SnapshotData::new(RSVIM_SNAPSHOT))?;

    // Initialize.
    event_loop.initialize()?;

    // Run loop.
    event_loop.run().await?;

    // Shutdown.
    event_loop.shutdown()?;

    std::process::exit(event_loop.exit_code);
  })
}
