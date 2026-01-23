use git2::Repository;
use rsvim_core::js::JsRuntimeForSnapshot;
use rsvim_core::js::v8_version;
use std::path::Path;

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
        Some(format!(
          "swc_core {}",
          core.unwrap().trim_start_matches("=")
        ))
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

  println!(
    "{LOG} Resolved version:{:?}, profile:{:?}, host:{:?}, git_commit:{:?}, swc_core:{:?}",
    version, profile, host, git_commit, swc_core
  );

  let git_commit = match git_commit {
    Some(git_commit) => format!("{}, ", git_commit),
    None => "".to_string(),
  };
  let v8 = format!("\nv8 {}", v8_version());
  let swc_core = match swc_core {
    Some(swc) => format!("\n{}", swc),
    None => "".to_string(),
  };
  let resolved = format!(
    "{} ({}{}, {}){}{}",
    version, git_commit, profile, host, v8, swc_core
  );

  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_VERSION.TXT");
  println!("{LOG} Writing version into {:?}...", output_path.as_path());

  std::fs::write(output_path.as_path(), resolved.as_bytes()).unwrap();
}

fn snapshot() {
  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  let snapshot = Box::from(&snapshot);
  let mut vec = Vec::with_capacity(snapshot.len());
  vec.extend_from_slice(&snapshot);

  let output_path =
    Path::new(env!("CARGO_MANIFEST_DIR")).join("RSVIM_SNAPSHOT.BIN");
  println!("{LOG} Writing snapshot into {:?}...", output_path.as_path());
  std::fs::write(output_path.as_path(), vec.into_boxed_slice()).unwrap();
}

fn main() {
  version();
  snapshot();
}
