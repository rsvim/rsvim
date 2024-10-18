use rsvim_core::js::{JsRuntimeForSnapshot, RSVIM_SNAPSHOT_BIN};
use std::path::Path;

fn main() {
  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  println!("snapshot is empty: {}", snapshot.is_empty());
  std::fs::write(Path::new(RSVIM_SNAPSHOT_BIN), &snapshot).unwrap();
}
