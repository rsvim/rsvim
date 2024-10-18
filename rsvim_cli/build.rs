use rsvim_core::js::JsRuntimeForSnapshot;
use std::path::Path;

fn main() {
  let js_runtime = JsRuntimeForSnapshot::new();
  eprintln!("[RSVIM] Build snapshot for rsvim cli...");
  let snapshot = {
    let snapshot = js_runtime.create_snapshot();
    let snapshot = Box::from(&snapshot);
    eprintln!(
      "[RSVIM] Snapshot blob size is {} before compress...",
      snapshot.len()
    );
    let mut vec = Vec::with_capacity(snapshot.len());
    vec.extend((snapshot.len() as u32).to_le_bytes());
    let max_compress_level: i32 = *zstd::compression_level_range().end();
    eprintln!(
      "[RSVIM] Compress snapshot with zstd-level={}...",
      max_compress_level
    );
    vec.extend_from_slice(
      &zstd::bulk::compress(&snapshot, max_compress_level)
        .expect("Failed to compress snapshot with zstd"),
    );
    let snapshot = vec.into_boxed_slice();
    eprintln!(
      "[RSVIM] Snapshot blob size is {} after compress...",
      snapshot.len()
    );
    snapshot
  };
  let output_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_SNAPSHOT.BIN");
  eprintln!("[RSVIM] Writing into fs {:?}...", output_path.as_path());
  std::fs::write(output_path.as_path(), &snapshot).unwrap();
}
