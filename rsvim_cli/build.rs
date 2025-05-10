use rsvim_core::js::{JsRuntimeForSnapshot, v8_version};
use std::io::Write;
use std::path::Path;

fn version() {
  let cargo_toml_src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml"));
  let cargo_toml_meta = cargo_toml_src.parse::<toml::Table>().unwrap();
  let output_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_VERSION.TXT");
  eprintln!(
    "[RSVIM] Writing version into {:?}...",
    output_path.as_path()
  );
  let rsvim_version = cargo_toml_meta["workspace"]["package"]["version"]
    .as_str()
    .unwrap();
  let mut f = std::fs::File::create(output_path).unwrap();
  write!(&mut f, "rsvim {} (v8 {})", rsvim_version, v8_version()).unwrap();
}

fn snapshot() {
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
  eprintln!(
    "[RSVIM] Writing snapshot into {:?}...",
    output_path.as_path()
  );
  std::fs::write(output_path.as_path(), &snapshot).unwrap();
}

fn main() {
  version();
  snapshot();
}
