use rsvim_core::js::JsRuntimeForSnapshot;

use std::path::Path;

fn main() {
  let mut js_runtime = JsRuntimeForSnapshot::new();
  let mut isolate = js_runtime.isolate.take().unwrap();
  let snapshot = isolate.create_blob(v8::FunctionCodeHandling::Keep).unwrap();
  eprintln!("snapshot is empty: {}", snapshot.is_empty());
  std::fs::write(Path::new("snapshot.bin"), &snapshot).unwrap();
}
