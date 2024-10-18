use rsvim_core::js::JsRuntimeForSnapshot;
use std::path::Path;

fn main() {
  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = {
    let snapshot = js_runtime.create_snapshot();
    let snapshot = Box::from(&snapshot);
    let mut vec = Vec::with_capacity(snapshot.len());
    vec.extend((snapshot.len() as u32).to_le_bytes());
    let max_compress_level: i32 = *zstd::compression_level_range().end();
    vec.extend_from_slice(
      &zstd::bulk::compress(&snapshot, max_compress_level)
        .expect("Failed to compress snapshot with zstd"),
    );
    vec.into_boxed_slice()
  };
  std::fs::write(Path::new("RSVIM_SNAPSHOT.BIN"), &snapshot).unwrap();
}
