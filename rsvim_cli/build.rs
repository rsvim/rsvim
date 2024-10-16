use rsvim_core::js::JsRuntimeForSnapshot;

use std::mem::ManuallyDrop;

fn main() {
  unsafe {
    let mut js_runtime = JsRuntimeForSnapshot::new();
    let isolate = ManuallyDrop::take(&mut js_runtime.isolate);
    let snapshot = isolate.create_blob(v8::FunctionCodeHandling::Keep).unwrap();
    std::fs::write("snapshot.bin", snapshot).unwrap();
  }
}
