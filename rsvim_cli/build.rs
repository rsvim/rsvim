use rsvim_core::js::JsRuntimeForSnapshot;

use std::path::Path;

fn main() {
  let mut js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  std::fs::write(Path::new("snapshot.bin"), &snapshot).unwrap();
}
