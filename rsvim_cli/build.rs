use rsvim_core::js::JsRuntime;

use std::path::Path;

fn main() {
  let mut js_runtime = JsRuntime::new_for_snapshot();
  let snapshot = js_runtime.create_snapshot();
  std::fs::write(Path::new("snapshot.bin"), &snapshot).unwrap();
}
