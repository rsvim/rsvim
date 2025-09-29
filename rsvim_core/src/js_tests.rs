use super::js::*;
use crate::prelude::*;
use assert_fs::prelude::PathChild;

#[test]
fn create_snapshot1() {
  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  let snapshot = Box::from(&snapshot);
  let mut vec = Vec::with_capacity(snapshot.len());
  vec.extend_from_slice(&snapshot);

  let temp_dir = assert_fs::TempDir::new().unwrap();
  let output_path = temp_dir.child("snapshot.bin");
  info!("Write snapshot to {:?}", output_path.path());
  std::fs::write(output_path.path(), vec.into_boxed_slice()).unwrap();
}
