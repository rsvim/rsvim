//! The VIM editor reinvented in Rust+TypeScript.
//!
//! See [rsvim_core] for more details.

use rsvim_core::cli::CliOpt;
use rsvim_core::evloop::EventLoop;
use rsvim_core::js::{SnapshotData, v8_version};
use rsvim_core::log;
use rsvim_core::prelude::*;

use std::path::{Path, PathBuf};
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

static RSVIM_HELP: &str = r#"The VIM editor reinvented in Rust+TypeScript

Usage: rsvim [FILE]...

Arguments:
  [FILE]...  Edit file(s)

Options:
  -V, --version  Print version
  -h, --help     Print help
"#;

fn parse_cli_args() -> Result<CliOpt, lexopt::Error> {
  use lexopt::prelude::*;

  // Arguments
  let mut file: Vec<PathBuf> = vec![];

  let mut parser = lexopt::Parser::from_env();
  while let Some(arg) = parser.next()? {
    match arg {
      Short('h') | Long("help") => {
        println!("{RSVIM_HELP}");
        std::process::exit(0);
      }
      Short('V') | Long("version") => {
        let pkg_version = env!("CARGO_PKG_VERSION");
        println!("rsvim {} (v8 {})", pkg_version, v8_version());
        std::process::exit(0);
      }
      Value(filename) => {
        file.push(Path::new(&filename).to_path_buf());
      }
      _ => return Err(arg.unexpected()),
    }
  }

  Ok(CliOpt { file })
}

fn main() -> IoResult<()> {
  log::init();
  let cli_opt = match parse_cli_args() {
    Ok(cli_opt) => cli_opt,
    Err(e) => {
      println!("error: {e}");
      println!("");
      println!("For more information, try '--help'");
      std::process::exit(0);
    }
  };
  trace!("cli_opt: {:?}", cli_opt);

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
