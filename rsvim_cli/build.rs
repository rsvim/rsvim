use git2::Repository;
use rsvim_core::js::JsRuntimeForSnapshot;
use rsvim_core::js::v8_version;
use std::path::Path;

fn version() {
  let profile = std::env::var("PROFILE").unwrap_or("debug".to_string());
  let opt_level = std::env::var("OPT_LEVEL").unwrap_or("0".to_string());
  let debug = std::env::var("DEBUG").unwrap_or("0".to_string());
  eprintln!(
    "[RSVIM] Env profile:{profile:?}, opt_level:{opt_level:?}, debug:{debug:?}..."
  );

  let version = if profile == "release"
    && (opt_level == "s" || opt_level == "z")
    && debug != "true"
  {
    format!("{} (v8 {})", env!("CARGO_PKG_VERSION"), v8_version())
  } else {
    let profile = if profile == "release" {
      "nightly".to_string()
    } else {
      profile
    };
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
  let snapshot = js_runtime.create_snapshot();
  let snapshot = Box::from(&snapshot);
  let mut vec = Vec::with_capacity(snapshot.len());
  vec.extend_from_slice(&snapshot);

  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_SNAPSHOT.BIN");
  eprintln!(
    "[RSVIM] Writing snapshot into {:?}...",
    output_path.as_path()
  );
  std::fs::write(output_path.as_path(), vec.into_boxed_slice()).unwrap();
}

fn main() {
  version();
  snapshot();
}
