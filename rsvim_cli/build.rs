use rsvim_core::js::{JsRuntimeForSnapshot, v8_version};
use std::io::Write;
use std::path::Path;

fn version() {
  // Emit vergen info.
  let cargo_vergen = vergen_git2::CargoBuilder::default()
    .target_triple(true)
    .build()
    .expect("Create vergen_git2::CargoBuilder failed");
  let git2_vergen = vergen_git2::Git2Builder::default()
    .commit_date(true)
    .build()
    .expect("Create vergen_git2::Git2Builder failed");

  vergen_git2::Emitter::default()
    .add_instructions(&cargo_vergen)
    .expect("Add vergen_git2::CargoBuilder failed")
    .add_instructions(&git2_vergen)
    .expect("Add vergen_git2::Git2Builder failed")
    .emit()
    .expect("Emit vergen_git2 failed");

  let output_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_VERSION.TXT");
  eprintln!(
    "[RSVIM] Writing version into {:?}...",
    output_path.as_path()
  );
  let pkg_version = env!("CARGO_PKG_VERSION");
  let git_commit_date = env!("VERGEN_GIT_COMMIT_DATE");
  let host_triple = env!("VERGEN_CARGO_TARGET_TRIPLE");
  let mut f = std::fs::File::create(output_path).unwrap();
  write!(
    &mut f,
    "rsvim {} (released at {}, build for {}, with v8 {})",
    pkg_version,
    git_commit_date,
    host_triple,
    v8_version()
  )
  .unwrap();
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
