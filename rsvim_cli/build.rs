use git2::Repository;
use rsvim_core::js::JsRuntimeForSnapshot;
use rsvim_core::js::v8_version;
use rsvim_core::prelude::*;

// pub const LOG: &str = "[RSVIM]";
pub const LOG: &str = "cargo:warning=[RSVIM]";

fn version() {
  let profile_env = std::env::var("PROFILE").unwrap_or("debug".to_string());
  let opt_level_env = std::env::var("OPT_LEVEL").unwrap_or("0".to_string());
  let debug_env = std::env::var("DEBUG").unwrap_or("0".to_string());
  let host = std::env::var("HOST").unwrap_or("unknown".to_string());
  println!(
    "{LOG} Env profile:{:?}, opt_level:{:?}, debug:{:?}, host:{:?}",
    profile_env, opt_level_env, debug_env, host
  );

  let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
  let version = env!("CARGO_PKG_VERSION").to_string();

  // profile
  let is_release_profile = profile_env == "release"
    && (opt_level_env == "s" || opt_level_env == "z")
    && debug_env != "true";
  let profile = if is_release_profile {
    "release".to_string()
  } else if profile_env == "release" {
    "nightly".to_string()
  } else {
    profile_env.clone()
  };

  // git commit
  let git_commit = match Repository::open(&workspace_dir) {
    Ok(repo) => {
      let head = repo.head().unwrap();
      let oid = head.target().unwrap();
      let commit = repo.find_commit(oid).unwrap();
      let id = commit.id();
      let id = id.to_string();
      println!("{LOG} Git id:{:?}", id);
      Some(id[0..8].to_string())
    }
    Err(e) => {
      println!("{LOG} Git error:{:?}", e);
      None
    }
  };

  // swc core
  let swc_core = match std::fs::read_to_string(workspace_dir.join("Cargo.toml"))
  {
    Ok(manifest) => match manifest.parse::<toml::Table>() {
      Ok(parsed_manifest) => {
        let deps = &parsed_manifest["workspace"]["dependencies"];
        let core = deps["swc_core"].as_str();
        println!("{LOG} Swc core:{:?}", core);
        Some(core.unwrap().trim_start_matches("=").to_string())
      }
      Err(e) => {
        println!("{LOG} Parse Cargo.toml error:{:?}", e);
        None
      }
    },
    Err(e) => {
      println!("{LOG} Read Cargo.toml error:{:?}", e);
      None
    }
  };
  let v8_version = v8_version();

  println!(
    "{LOG} Resolved version:{:?}, profile:{:?}, host:{:?}, git_commit:{:?}, v8:{:?}, swc_core:{:?}",
    version, profile, host, git_commit, v8_version, swc_core
  );

  let mut resolved = format!(
    "version={}\nprofile={}\nhost={}\nv8={}\n",
    version, profile, host, v8_version
  );
  if let Some(git_commit) = git_commit {
    resolved = format!("{}git_commit={}\n", resolved, git_commit);
  }
  if let Some(swc_core) = swc_core {
    resolved = format!("{}swc_core={}\n", resolved, swc_core);
  }

  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_VERSION.TXT");
  println!("{LOG} Writing version into {:?}...", output_path.as_path());

  std::fs::write(output_path.as_path(), resolved.as_bytes()).unwrap();
}

fn snapshot() {
  let snapshot = {
    let js_runtime = JsRuntimeForSnapshot::new();
    let snapshot = js_runtime.create_snapshot();
    let snapshot = Box::from(&snapshot);
    let snapshot_len = snapshot.len();
    println!("{LOG} Snapshot is {snapshot_len} bytes before compress...");
    let mut vec = Vec::with_capacity(snapshot.len());
    vec.extend((snapshot.len() as u32).to_le_bytes());
    let max_compress_level: i32 = *zstd::compression_level_range().end();
    println!("{LOG} Compress snapshot with zstd-level={max_compress_level}...");
    vec.extend_from_slice(
      &zstd::bulk::compress(&snapshot, max_compress_level)
        .expect("Failed to compress snapshot with zstd"),
    );
    let snapshot = vec.into_boxed_slice();
    let snapshot_len = snapshot.len();
    println!("{LOG} Snapshot is {snapshot_len} bytes after compress...");
    snapshot
  };

  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_SNAPSHOT.BIN");
  println!("{LOG} Writing snapshot into {:?}...", output_path.as_path());
  std::fs::write(output_path.as_path(), &snapshot).unwrap();
}

fn main() {
  version();
  snapshot();
}
