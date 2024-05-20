//! The VIM editor reinvented in Rust+Typescript.

use clap::Parser;
use rsvim::{cli, log, ui};
use tracing::debug;
// use heed::types as heed_types;
// use heed::{byteorder, Database, EnvOpenOptions};

#[tokio::main]
async fn main() -> std::io::Result<()> {
  let cli = cli::Cli::parse();
  log::init(&cli);
  debug!("cli: {:?}", cli);

  // let dir = tempfile::tempdir().unwrap();
  // debug!("tempdir:{:?}", dir);
  // let env = unsafe { EnvOpenOptions::new().open(dir.path()).unwrap() };
  // let mut wtxn = env.write_txn().unwrap();
  // let db: Database<heed_types::Str, heed_types::U32<byteorder::NativeEndian>> =
  //   env.create_database(&mut wtxn, None).unwrap();
  // db.put(&mut wtxn, "seven", &7).unwrap();
  // wtxn.commit().unwrap();

  let mut t = ui::term::Terminal::init().await?;
  t.run().await?;
  t.shutdown().await
}
