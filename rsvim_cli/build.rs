use git2::Repository;
use rsvim_core::js::{JsRuntimeForSnapshot, v8_version};
use std::path::Path;

fn version() {
  let profile = std::env::var("PROFILE").unwrap_or("release");

  let profile = if profile == "release" {
    "release"
  } else {
    "dev"
  };

  let version = if profile == "release" {
    format!("{} (v8 {})", env!("CARGO_PKG_VERSION"), v8_version())
  } else {
    let repo_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let maybe_git_commit = match Repository::open(repo_path) {
      Ok(repo) => {
        let head = repo.head().unwrap();
        let oid = head.target().unwrap();
        let commit = repo.find_commit(oid).unwrap();
        let id = commit.id();
        let id = id.to_string();
        format!("+{}", &id[0..8])
      }
      Err(_) => "".to_string(),
    };

    format!(
      "{}+{}{} (v8 {})",
      env!("CARGO_PKG_VERSION"),
      profile,
      maybe_git_commit,
      v8_version()
    )
  };

  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_VERSION.TXT");
  eprintln!(
    "[RSVIM] Writing version into {:?}...",
    output_path.as_path()
  );

  std::fs::write(output_path.as_path(), version.as_bytes()).unwrap();
}

fn snapshot() {
  let js_runtime = JsRuntimeForSnapshot::new();
  eprintln!("[RSVIM] Build snapshot for rsvim cli...");
  let snapshot = {
    let snapshot = js_runtime.create_snapshot();
    let snapshot = Box::from(&snapshot);
    let snapshot_len = snapshot.len();
    eprintln!(
      "[RSVIM] Snapshot blob size is {snapshot_len} before compress..."
    );
    let mut vec = Vec::with_capacity(snapshot.len());
    vec.extend((snapshot.len() as u32).to_le_bytes());
    let max_compress_level: i32 = *zstd::compression_level_range().end();
    eprintln!(
      "[RSVIM] Compress snapshot with zstd-level={max_compress_level}..."
    );
    vec.extend_from_slice(
      &zstd::bulk::compress(&snapshot, max_compress_level)
        .expect("Failed to compress snapshot with zstd"),
    );
    let snapshot = vec.into_boxed_slice();
    let snapshot_len = snapshot.len();
    eprintln!("[RSVIM] Snapshot blob size is {snapshot_len} after compress...");
    snapshot
  };
  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_SNAPSHOT.BIN");
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
