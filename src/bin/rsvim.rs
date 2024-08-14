//! The VIM editor reinvented in Rust+Typescript.

use clap::Parser;
use rsvim::{cli, log};
// use heed::types as heed_types;
// use heed::{byteorder, Database, EnvOpenOptions};
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{execute, terminal};
use rsvim::evloop::EventLoop;
use std::io::Result as IoResult;
use tracing::debug;

pub async fn init() -> IoResult<()> {
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

pub async fn shutdown() -> IoResult<()> {
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

#[tokio::main]
async fn main() -> IoResult<()> {
  log::init();

  let cli = cli::Cli::parse();
  debug!("cli: {:?}", cli);

  // let dir = tempfile::tempdir().unwrap();
  // debug!("tempdir:{:?}", dir);
  // let env = unsafe { EnvOpenOptions::new().open(dir.path()).unwrap() };
  // let mut wtxn = env.write_txn().unwrap();
  // let db: Database<heed_types::Str, heed_types::U32<byteorder::NativeEndian>> =
  //   env.create_database(&mut wtxn, None).unwrap();
  // db.put(&mut wtxn, "seven", &7).unwrap();
  // wtxn.commit().unwrap();

  init().await?;

  // // V8 engine
  // let v8_platform = v8::new_default_platform(0, false).make_shared();
  // v8::V8::initialize_platform(v8_platform);
  // v8::V8::initialize();
  // let v8_isolate = &mut v8::Isolate::new(Default::default());
  // let v8_handle_scope = &mut v8::HandleScope::new(v8_isolate);
  // let v8_context = v8::Context::new(v8_handle_scope, Default::default());
  // let v8_context_scope = &mut v8::ContextScope::new(v8_handle_scope, v8_context);
  // let js_code = v8::String::new(v8_context_scope, "'Hello' + ' World!'").unwrap();
  // debug!(
  //   "javascript code: {}",
  //   js_code.to_rust_string_lossy(v8_context_scope)
  // );
  // let v8_script = v8::Script::compile(v8_context_scope, js_code, None).unwrap();
  // let js_result = v8_script.run(v8_context_scope).unwrap();
  // let js_result = js_result.to_string(v8_context_scope).unwrap();
  // debug!(
  //   "javascript result: {}",
  //   js_result.to_rust_string_lossy(v8_context_scope)
  // );

  // Event loop
  let mut event_loop = EventLoop::new().await?;
  event_loop.init().await?;
  event_loop.run().await?;

  shutdown().await
}
