use rsvim_core::js::JsRuntimeForSnapshot;
use std::path::Path;

fn main() {
  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  println!("snapshot is empty: {}", snapshot.is_empty());
  std::fs::write(Path::new("RSVIM_SNAPSHOT.BIN"), &snapshot).unwrap();
}
