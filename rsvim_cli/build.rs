use rsvim_core::js::JsRuntimeForSnapshot;

use std::path::Path;

const RSVIM_SNAPSHOT: &str = "rsvim_snapshot.bin";

fn main() {
  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  println!("snapshot is empty: {}", snapshot.is_empty());
  std::fs::write(Path::new(RSVIM_SNAPSHOT), &snapshot).unwrap();
}
