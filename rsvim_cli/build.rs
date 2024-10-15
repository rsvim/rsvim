use rsvim_core::js::{JsRuntimeForSnapshot, JsRuntimeOptions};

fn main() {
  let mut js_runtime = JsRuntimeForSnapshot::new(JsRuntimeOptions::default());
  js_runtime.init_environment();
  let snapshot = js_runtime
    .isolate
    .create_blob(v8::FunctionCodeHandling::Keep)
    .unwrap();
  std::fs::write("snapshot.bin", snapshot).unwrap();
}
